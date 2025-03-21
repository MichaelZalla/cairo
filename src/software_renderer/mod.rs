use std::{cell::RefCell, f32::EPSILON, rc::Rc};

#[cfg(feature = "debug_cycle_counts")]
use profile::SoftwareRendererCycleCounter;

use crate::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::Color,
    geometry::primitives::{aabb::AABB, ray::Ray},
    material::Material,
    matrix::Mat4,
    render::{
        options::{shader::RenderShaderOptions, RenderOptions, RenderPassFlag},
        viewport::RenderViewport,
        Renderer,
    },
    resource::{arena::Arena, handle::Handle},
    scene::{
        camera::{frustum::Frustum, Camera},
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
        resources::SceneResources,
    },
    shader::{
        alpha::AlphaShaderFn,
        context::ShaderContext,
        fragment::FragmentShaderFn,
        geometry::{sample::GeometrySample, GeometryShaderFn},
        vertex::VertexShaderFn,
    },
    shaders::{
        default_alpha_shader::DEFAULT_ALPHA_SHADER,
        default_geometry_shader::DEFAULT_GEOMETRY_SHADER,
    },
    stats::CycleCounters,
    texture::{cubemap::CubeMap, map::TextureMap},
    transform::quaternion::Quaternion,
    vec::vec4::Vec4,
    vertex::default_vertex_out::DefaultVertexOut,
};

use self::gbuffer::GBuffer;

use super::{mesh::Mesh, vec::vec3::Vec3};

use pass::ssao_pass::{make_4x4_tangent_space_rotations, make_hemisphere_kernel, KERNEL_SIZE};

mod gbuffer;
mod pass;
mod primitive;
mod profile;

pub mod zbuffer;

#[derive(Debug, Clone)]
pub struct SoftwareRenderer {
    pub options: RenderOptions,
    pub cycle_counters: CycleCounters,
    pub shader_options: RenderShaderOptions,
    framebuffer: Option<Rc<RefCell<Framebuffer>>>,
    viewport: RenderViewport,
    g_buffer: Option<GBuffer>,
    alpha_accumulation_buffer: Buffer2D<Vec4>,
    alpha_revealage_buffer: Buffer2D<f32>,
    pub ssao_buffer: Option<TextureMap<f32>>,
    ssao_blur_buffer: Option<TextureMap<f32>>,
    ssao_hemisphere_kernel: Option<[Vec3; KERNEL_SIZE]>,
    ssao_4x4_tangent_space_rotations: Option<[Quaternion; 16]>,
    pub shader_context: Rc<RefCell<ShaderContext>>,
    scene_resources: Rc<SceneResources>,
    vertex_shader: VertexShaderFn,
    alpha_shader: AlphaShaderFn,
    geometry_shader: GeometryShaderFn,
    fragment_shader: FragmentShaderFn,
}

impl Renderer for SoftwareRenderer {
    fn get_options(&self) -> &RenderOptions {
        &self.options
    }

    fn get_options_mut(&mut self) -> &mut RenderOptions {
        &mut self.options
    }

    fn begin_frame(&mut self) {
        #[cfg(feature = "debug_cycle_counts")]
        {
            self.cycle_counters.reset();

            self.cycle_counters
                .get_mut(SoftwareRendererCycleCounter::BeginAndEndFrame as usize)
                .start();
        }

        // Clear the bound framebuffer.

        if let Some(rc) = &self.framebuffer {
            let mut framebuffer = rc.borrow_mut();

            framebuffer.clear();
        }

        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Rasterization | RenderPassFlag::DeferredLighting)
        {
            // Clear the geometry buffer (resets stencil bits).

            if let Some(g_buffer) = self.g_buffer.as_mut() {
                g_buffer.clear();
            }

            // Clear the accumulation buffer.

            self.alpha_accumulation_buffer.clear(None);

            // Clear the revealage buffer.

            self.alpha_revealage_buffer.clear(Some(1.0));

            // Clear the SSAO buffer.

            if let Some(ssao_buffer) = self.ssao_buffer.as_mut() {
                let map = &mut ssao_buffer.levels[0];

                map.0.clear(None);
            }

            // Clear the SSAO blur buffer.

            if let Some(ssao_blur_buffer) = self.ssao_blur_buffer.as_mut() {
                let map = &mut ssao_blur_buffer.levels[0];

                map.0.clear(None);
            }
        }
    }

    fn end_frame(&mut self) {
        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Rasterization | RenderPassFlag::DeferredLighting)
        {
            // Approximate screen-space ambient occlusion pass.

            if self
                .options
                .render_pass_flags
                .contains(RenderPassFlag::Ssao)
            {
                self.do_ssao_pass();
            }

            // Deferred lighting pass.

            self.do_deferred_lighting_pass();

            // Semi-transparent fragment pass.

            self.do_weighted_blended_pass();

            // Bloom pass (with or without dirt mask).

            if self
                .options
                .render_pass_flags
                .contains(RenderPassFlag::Bloom)
            {
                self.do_bloom_pass();
            }
        }

        // Tone-mapping pass (or basic blit).

        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::ToneMapping)
        {
            self.do_tone_mapping_pass();
        } else if let Some(framebuffer_rc) = &self.framebuffer {
            let framebuffer = framebuffer_rc.borrow();

            // Blit the forward color buffer.

            if let Some(deferred_buffer_rc) = framebuffer.attachments.deferred_hdr.as_ref() {
                let mut deferred_buffer = deferred_buffer_rc.borrow_mut();

                for color_hdr in deferred_buffer.iter_mut() {
                    *color_hdr = color_hdr.clamp(0.0, 1.0);
                }
            }
        }

        // Combine the forward and deferred (HDR) color buffers into the default
        // color buffer.

        if let Some(framebuffer_rc) = &self.framebuffer {
            let framebuffer = framebuffer_rc.borrow();

            // Blit the forward color buffer.

            if let (Some(forward_buffer_rc), Some(color_buffer_rc)) = (
                framebuffer.attachments.forward_ldr.as_ref(),
                framebuffer.attachments.color.as_ref(),
            ) {
                let forward_buffer = forward_buffer_rc.borrow();

                let mut color_buffer = color_buffer_rc.borrow_mut();

                let forward_fragments = forward_buffer.get_all();

                // Skips pixels in our forward buffer if they weren't written to.
                for (index, value) in forward_fragments.iter().enumerate() {
                    if Color::from_u32(*value).a > 0.0 {
                        color_buffer.set_at(index, *value);
                    }
                }
            }
        }

        #[cfg(feature = "debug_cycle_counts")]
        {
            self.cycle_counters
                .get_mut(SoftwareRendererCycleCounter::BeginAndEndFrame as usize)
                .end();

            self.cycle_counters.report::<SoftwareRendererCycleCounter>();
        }
    }

    fn render_point(
        &mut self,
        point_world_space: Vec3,
        color: Color,
        camera: Option<&Camera>,
        materials: Option<&mut Arena<Material>>,
        material: Option<Handle>,
        scale: Option<f32>,
    ) {
        self._render_point(point_world_space, color, camera, materials, material, scale)
    }

    fn render_line(&mut self, start_world_space: Vec3, end_world_space: Vec3, color: Color) {
        self._render_line(start_world_space, end_world_space, color)
    }

    fn render_circle(&mut self, position: &Vec3, radius_world_units: f32, color: Color) {
        self._render_circle(position, radius_world_units, color)
    }

    fn render_axes(&mut self, position: Option<Vec3>, scale: Option<f32>) {
        self._render_axes(position, scale)
    }

    fn render_ground_plane(&mut self, scale: f32) {
        self._render_ground_plane(scale)
    }

    fn render_frustum(&mut self, frustum: &Frustum, color: Option<Color>) {
        self._render_frustum(frustum, color)
    }

    fn render_camera(&mut self, camera: &Camera, color: Option<Color>) {
        self._render_camera(camera, color)
    }

    fn render_ambient_light(&mut self, transform: &Mat4, light: &AmbientLight) {
        self._render_ambient_light(transform, light)
    }

    fn render_directional_light(&mut self, transform: &Mat4, light: &DirectionalLight) {
        self._render_directional_light(transform, light)
    }

    fn render_point_light(&mut self, transform: &Mat4, light: &PointLight) {
        self._render_point_light(transform, light)
    }

    fn render_spot_light(&mut self, transform: &Mat4, light: &SpotLight) {
        self._render_spot_light(transform, light)
    }

    fn render_ray(&mut self, ray: &Ray, color: Color) {
        self._render_ray(ray, color)
    }

    fn render_aabb(&mut self, aabb: &AABB, world_transform: &Mat4, color: Color) {
        self._render_aabb(aabb, world_transform, color)
    }

    fn render_entity(
        &mut self,
        world_transform: &Mat4,
        culling_frustum: &Option<Frustum>,
        entity_mesh: &Mesh,
        entity_material: &Option<Handle>,
    ) -> bool {
        self._render_entity(
            world_transform,
            culling_frustum,
            entity_mesh,
            entity_material,
        )
    }

    fn render_skybox(&mut self, skybox: &CubeMap, camera: &Camera, skybox_rotation: Option<Mat4>) {
        self._render_skybox(skybox, camera, skybox_rotation)
    }

    fn render_skybox_hdr(
        &mut self,
        skybox_hdr: &CubeMap<Vec3>,
        camera: &Camera,
        skybox_rotation: Option<Mat4>,
    ) {
        self._render_skybox_hdr(skybox_hdr, camera, skybox_rotation)
    }
}

impl SoftwareRenderer {
    pub fn new(
        shader_context: Rc<RefCell<ShaderContext>>,
        scene_resources: Rc<SceneResources>,
        vertex_shader: VertexShaderFn,
        fragment_shader: FragmentShaderFn,
        options: RenderOptions,
    ) -> Self {
        let alpha_shader = DEFAULT_ALPHA_SHADER;

        let geometry_shader = DEFAULT_GEOMETRY_SHADER;

        let shader_options: RenderShaderOptions = Default::default();

        let framebuffer = None;

        let viewport: RenderViewport = Default::default();

        SoftwareRenderer {
            options,
            cycle_counters: Default::default(),
            framebuffer,
            viewport,
            g_buffer: None,
            ssao_buffer: None,
            ssao_blur_buffer: None,
            ssao_hemisphere_kernel: None,
            ssao_4x4_tangent_space_rotations: None,
            alpha_accumulation_buffer: Default::default(),
            alpha_revealage_buffer: Default::default(),
            shader_context,
            scene_resources,
            vertex_shader,
            alpha_shader,
            geometry_shader,
            shader_options,
            fragment_shader,
        }
    }

    pub fn set_vertex_shader(&mut self, shader: VertexShaderFn) {
        self.vertex_shader = shader;
    }

    pub fn set_geometry_shader(&mut self, shader: GeometryShaderFn) {
        self.geometry_shader = shader;
    }

    pub fn set_fragment_shader(&mut self, shader: FragmentShaderFn) {
        self.fragment_shader = shader;
    }

    pub fn bind_framebuffer(&mut self, framebuffer_option: Option<Rc<RefCell<Framebuffer>>>) {
        match &framebuffer_option {
            Some(framebuffer_rc) => {
                let framebuffer = framebuffer_rc.borrow();

                let (width, height) = (framebuffer.width, framebuffer.height);

                match framebuffer.validate() {
                    Ok(()) => {
                        self.framebuffer.clone_from(&framebuffer_option);

                        self.viewport = RenderViewport::from_framebuffer(&framebuffer);

                        let should_reallocate_g_buffer = match &self.g_buffer {
                            Some(g_buffer) => {
                                g_buffer.0.width != width || g_buffer.0.height != height
                            }
                            None => true,
                        };

                        self.alpha_accumulation_buffer.resize(width, height);

                        self.alpha_revealage_buffer.resize(width, height);

                        let should_reallocate_ssao_buffers = match &self.ssao_buffer {
                            Some(ssao_buffer) => {
                                ssao_buffer.width != width || ssao_buffer.height != height
                            }
                            None => true,
                        };

                        if should_reallocate_g_buffer {
                            // Re-allocate a G-buffer.

                            self.g_buffer = Some(GBuffer::new(width, height));
                        }

                        if should_reallocate_ssao_buffers {
                            // Re-allocate an SSAO buffer.

                            let buffer = Buffer2D::<f32>::new(width, height, None);

                            self.ssao_buffer
                                .replace(TextureMap::from_buffer(width, height, buffer));

                            self.ssao_blur_buffer.clone_from(&self.ssao_buffer);

                            self.ssao_hemisphere_kernel
                                .replace(make_hemisphere_kernel());

                            self.ssao_4x4_tangent_space_rotations
                                .replace(make_4x4_tangent_space_rotations());
                        }
                    }
                    Err(err) => {
                        panic!("Called Renderer::bind_framebuffer() with an invalid Framebuffer! (Err: {})", err);
                    }
                }
            }
            None => {
                self.framebuffer = None;
                self.g_buffer = None;
                self.ssao_buffer = None;
                self.ssao_blur_buffer = None;
            }
        }
    }

    fn submit_fragment(&mut self, x: u32, y: u32, interpolant: &mut DefaultVertexOut) {
        let shader_context = self.shader_context.borrow();

        let framebuffer = self.framebuffer.as_ref().unwrap().borrow();

        let mut depth_buffer = framebuffer.attachments.depth.as_ref().unwrap().borrow_mut();

        let mut stencil_buffer = framebuffer
            .attachments
            .stencil
            .as_ref()
            .unwrap()
            .borrow_mut();

        // Restore linear space interpolant.

        let mut linear_space_interpolant =
            *interpolant * (1.0 / interpolant.position_projection_space.w);

        let linear_space_z = linear_space_interpolant.position_projection_space.z;

        if let Some(((x, y), non_linear_z)) = depth_buffer.test(x, y, linear_space_z) {
            // Alpha shader test.

            if !(self.alpha_shader)(
                &shader_context,
                &self.scene_resources,
                &linear_space_interpolant,
            ) {
                return;
            }

            // Geometry shader.

            linear_space_interpolant.depth = depth_buffer.get_normalized(linear_space_z);

            if let Some(sample) = (self.geometry_shader)(
                &shader_context,
                &self.scene_resources,
                &self.shader_options,
                &linear_space_interpolant,
            ) {
                // Opaque vs. semi-transparent paths.

                if sample.alpha > 1.0 - EPSILON {
                    // Write non-linear depth to the depth buffer.

                    depth_buffer.set(x, y, non_linear_z);

                    // Write to the stencil buffer.

                    stencil_buffer.set(x, y);

                    // Write to either the geometry buffer or the forward color buffer.

                    if self
                        .options
                        .render_pass_flags
                        .contains(RenderPassFlag::DeferredLighting)
                    {
                        if let Some(g_buffer) = self.g_buffer.as_mut() {
                            g_buffer.set(x, y, sample);
                        }
                    } else if let Some(forward_buffer_rc) =
                        framebuffer.attachments.forward_ldr.as_ref()
                    {
                        let mut forward_buffer = forward_buffer_rc.borrow_mut();

                        let hdr_color = self.get_hdr_color_for_sample(
                            &shader_context,
                            &self.scene_resources,
                            &sample,
                        );

                        let ldr_color = self.get_ldr_color(hdr_color);

                        let ldr_color_u32 = ldr_color.to_u32();

                        forward_buffer.set(x, y, ldr_color_u32);
                    }
                } else {
                    // Skip writing to the depth buffer.

                    let (accumulation, revealage) =
                        self.get_accumulation_and_revealage(&shader_context, sample);

                    //  Write to the (color) accumulation buffer.

                    let src = accumulation;
                    let dest = *self.alpha_accumulation_buffer.get(x, y);

                    // Source: GL_ONE, dest: GL_ONE
                    let blended = dest + src; // 1.0 * dest + 1.0 * src

                    self.alpha_accumulation_buffer.set(x, y, blended);

                    //  Write to the (alpha) revealage buffer.

                    let src = revealage;
                    let dest = *self.alpha_revealage_buffer.get(x, y);

                    // Source: GL_ZERO, dest: GL_ONE_MINUS_SRC_ALPHA

                    let blended = (1.0 - src) * dest; // + 0.0 * src

                    self.alpha_revealage_buffer.set(x, y, blended);
                }
            }
        }
    }

    fn get_accumulation_and_revealage(
        &self,
        shader_context: &ShaderContext,
        sample: GeometrySample,
    ) -> (Vec4, f32) {
        let hdr_color =
            self.get_hdr_color_for_sample(shader_context, &self.scene_resources, &sample);

        let alpha = sample.alpha;

        let depth_abs = sample.depth.abs();

        let weight = alpha * (1.0 - depth_abs * depth_abs * depth_abs);

        let accumulation = Vec4::new(hdr_color * alpha, alpha) * weight;

        let revealage = alpha;

        (accumulation, revealage)
    }

    fn get_ldr_color(&self, hdr_color: Vec3) -> Color {
        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::ToneMapping)
        {
            self.get_tone_mapped_color_from_hdr(hdr_color)
        } else {
            Color::from_vec3(hdr_color.clamp(0.0, 1.0))
        }
    }

    fn get_hdr_color_for_sample(
        &self,
        shader_context: &ShaderContext,
        scene_resources: &SceneResources,
        sample: &GeometrySample,
    ) -> Vec3 {
        let render_pass_flags = self.get_options().render_pass_flags;

        if render_pass_flags.contains(RenderPassFlag::Lighting) {
            (self.fragment_shader)(shader_context, scene_resources, sample)
        } else if render_pass_flags.contains(RenderPassFlag::Ssao) {
            sample.albedo * sample.ambient_factor
        } else {
            sample.albedo
        }
    }

    fn get_tone_mapped_color_from_hdr(&self, color_hdr: Vec3) -> Color {
        let mut tone_mapped = self.options.tone_mapping.map(color_hdr);

        // Gamma correct: Transforms linear space to sRGB space.

        tone_mapped.linear_to_srgb();

        Color::from_vec3(tone_mapped * 255.0)
    }
}

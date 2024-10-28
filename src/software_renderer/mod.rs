use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "debug_cycle_counts")]
use profile::SoftwareRendererCycleCounter;

use crate::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::Color,
    effects::gaussian_blur::GaussianBlurEffect,
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::{geometry::Geometry, Face},
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
    texture::cubemap::CubeMap,
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

use self::gbuffer::GBuffer;

use super::{mesh::Mesh, vec::vec3::Vec3};

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
    bloom_buffer: Option<Buffer2D<Vec3>>,
    bloom_effect: Option<GaussianBlurEffect>,
    pub shader_context: Rc<RefCell<ShaderContext>>,
    scene_resources: Rc<SceneResources>,
    vertex_shader: VertexShaderFn,
    alpha_shader: AlphaShaderFn,
    geometry_shader: GeometryShaderFn,
    fragment_shader: FragmentShaderFn,
}

impl Renderer for SoftwareRenderer {
    fn begin_frame(&mut self) {
        #[cfg(feature = "debug_cycle_counts")]
        {
            self.cycle_counters.reset();

            self.cycle_counters
                .get_mut(SoftwareRendererCycleCounter::BeginAndEndFrame as usize)
                .start();
        }

        if let Some(rc) = &self.framebuffer {
            let mut framebuffer = rc.borrow_mut();

            framebuffer.clear();
        }

        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Rasterization)
        {
            if let Some(g_buffer) = self.g_buffer.as_mut() {
                g_buffer.clear();
            }

            if let Some(bloom_buffer) = self.bloom_buffer.as_mut() {
                bloom_buffer.clear(None);
            }
        }
    }

    fn end_frame(&mut self) {
        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Rasterization | RenderPassFlag::DeferredLighting)
        {
            // Deferred lighting.

            self.do_deferred_lighting_pass();

            // Bloom pass over the deferred (HDR) buffer.

            if self
                .options
                .render_pass_flags
                .contains(RenderPassFlag::Bloom)
            {
                self.do_bloom_pass();
            }
        }

        // Blit deferred (HDR) framebuffer to the (final) color framebuffer.

        if let Some(rc) = &self.framebuffer {
            let framebuffer = rc.borrow_mut();

            if self
                .options
                .render_pass_flags
                .contains(RenderPassFlag::Rasterization)
            {
                if let (Some(color_buffer_rc), Some(deferred_buffer_rc)) = (
                    framebuffer.attachments.color.as_ref(),
                    framebuffer.attachments.forward_or_deferred_hdr.as_ref(),
                ) {
                    let (mut color_buffer, deferred_buffer) =
                        (color_buffer_rc.borrow_mut(), deferred_buffer_rc.borrow());

                    for y in 0..framebuffer.height {
                        for x in 0..framebuffer.width {
                            let lit_geometry_fragment_color_tone =
                                self.get_tone_mapped_color_from_hdr(*deferred_buffer.get(x, y));

                            color_buffer.set(x, y, lit_geometry_fragment_color_tone.to_u32());
                        }
                    }
                }
            }

            if let (Some(color_buffer_rc), Some(forward_buffer_rc)) = (
                framebuffer.attachments.color.as_ref(),
                framebuffer.attachments.forward_ldr.as_ref(),
            ) {
                let (mut color_buffer, forward_buffer) =
                    (color_buffer_rc.borrow_mut(), forward_buffer_rc.borrow());

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

    fn render_point_indicator(&mut self, position: Vec3, scale: f32) {
        self._render_point_indicator(position, scale)
    }

    fn render_world_axes(&mut self, scale: f32) {
        self._render_world_axes(scale)
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

    fn render_entity_aabb(
        &mut self,
        entity: &Entity,
        world_transform: &Mat4,
        mesh_arena: &Arena<Mesh>,
        wireframe_color: &Vec3,
    ) {
        self._render_entity_aabb(entity, world_transform, mesh_arena, wireframe_color)
    }

    fn render_entity(
        &mut self,
        world_transform: &Mat4,
        clipping_camera_frustum: &Option<Frustum>,
        entity_mesh: &Mesh,
        entity_material: &Option<Handle>,
    ) -> bool {
        self._render_entity(
            world_transform,
            clipping_camera_frustum,
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
            bloom_buffer: None,
            bloom_effect: None,
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
                let refcell = &**framebuffer_rc;
                let framebuffer = refcell.borrow();

                match framebuffer.validate() {
                    Ok(()) => {
                        self.framebuffer.clone_from(&framebuffer_option);

                        self.viewport = RenderViewport::from_framebuffer(&framebuffer);

                        let should_reallocate_g_buffer = match &self.g_buffer {
                            Some(g_buffer) => {
                                g_buffer.0.width != framebuffer.width
                                    || g_buffer.0.height != framebuffer.height
                            }
                            None => true,
                        };

                        let should_reallocate_bloom_buffer = match &self.bloom_buffer {
                            Some(bloom_buffer) => {
                                bloom_buffer.width != framebuffer.width
                                    || bloom_buffer.height != framebuffer.height
                            }
                            None => true,
                        };

                        if should_reallocate_g_buffer {
                            // Re-allocate a G-buffer.

                            self.g_buffer =
                                Some(GBuffer::new(framebuffer.width, framebuffer.height));
                        }

                        if should_reallocate_bloom_buffer {
                            // Re-allocate a bloom buffer.

                            self.bloom_buffer = Some(Buffer2D::<Vec3>::new(
                                framebuffer.width,
                                framebuffer.height,
                                None,
                            ));
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
                self.bloom_buffer = None;
            }
        }
    }

    fn render_entity_mesh(&mut self, mesh: &Mesh, world_transform: &Mat4) {
        // Otherwise, cull individual triangles.

        let original_world_transform: Mat4;

        {
            let mut context = self.shader_context.borrow_mut();

            original_world_transform = context.get_world_transform();

            context.set_world_transform(*world_transform);
        }

        let geometry = mesh.geometry.as_ref();

        self.render_mesh_geometry(geometry, &mesh.faces);

        // Reset the shader context's original world transform.
        {
            let mut context = self.shader_context.borrow_mut();

            context.set_world_transform(original_world_transform);
        }
    }

    fn render_mesh_geometry(&mut self, geometry: &Geometry, faces: &[Face]) {
        self.process_object_space_vertices(geometry, faces);
    }

    fn process_object_space_vertices(&mut self, geometry: &Geometry, faces: &[Face]) {
        // Map each face to a set of 3 unique instances of DefaultVertexIn.

        let mut vertices_in: Vec<DefaultVertexIn> = Vec::with_capacity(faces.len() * 3);

        for face in faces {
            let [v0_in, v1_in, v2_in] = get_vertices_in(geometry, face);

            vertices_in.push(v0_in);
            vertices_in.push(v1_in);
            vertices_in.push(v2_in);
        }

        // Process mesh vertices from object-space to world-space.
        let projection_space_vertices: Vec<DefaultVertexOut>;

        {
            let shader_context = self.shader_context.borrow();

            projection_space_vertices = vertices_in
                .into_iter()
                .map(|v_in| (self.vertex_shader)(&shader_context, &v_in))
                .collect();
        }

        self.process_triangles(faces, projection_space_vertices.as_slice());
    }

    fn transform_to_ndc_space(&mut self, v: &mut DefaultVertexOut) {
        let w_inverse = 1.0 / v.position_projection_space.w;

        *v *= w_inverse;

        v.position_projection_space.x = (v.position_projection_space.x + 1.0) * self.viewport.width_over_2;
        v.position_projection_space.y = (-v.position_projection_space.y + 1.0) * self.viewport.height_over_2;

        v.position_projection_space.w = w_inverse;
    }

    fn test_and_set_z_buffer(
        &mut self,
        x: u32,
        y: u32,
        interpolant: &mut DefaultVertexOut,
        // shader_context: &ShaderContext,
        // scene_resources: &SceneResources,
    ) {
        match &self.framebuffer {
            Some(rc) => {
                let framebuffer = rc.borrow_mut();

                match framebuffer.attachments.depth.as_ref() {
                    Some(depth_buffer_rc) => {
                        let mut depth_buffer = depth_buffer_rc.borrow_mut();

                        // Restore linear space interpolant.

                        let mut linear_space_interpolant =
                            *interpolant * (1.0 / interpolant.position_projection_space.w);

                        if let Some(((x, y), non_linear_z)) =
                            depth_buffer.test(x, y, linear_space_interpolant.position_projection_space.z)
                        {
                            // Alpha shader test.

                            let shader_context = self.shader_context.borrow();

                            if !(self.alpha_shader)(
                                &shader_context,
                                &self.scene_resources,
                                &linear_space_interpolant,
                            ) {
                                return;
                            }

                            // Geometry shader.

                            if let Some(g_buffer) = self.g_buffer.as_mut() {
                                let z = linear_space_interpolant.position_projection_space.z;
                                let near = depth_buffer.get_projection_z_near();
                                let far = depth_buffer.get_projection_z_far();

                                linear_space_interpolant.depth =
                                    ((z - near) / (far - near)).clamp(0.0, 1.0);

                                if let Some(sample) = (self.geometry_shader)(
                                    &shader_context,
                                    &self.scene_resources,
                                    &self.shader_options,
                                    &linear_space_interpolant,
                                ) {
                                    // Write to the depth attachment.

                                    depth_buffer.set(x, y, non_linear_z);

                                    if let Some(stencil_buffer_rc) =
                                        framebuffer.attachments.stencil.as_ref()
                                    {
                                        // Write to the stencil attachment.

                                        let mut stencil_buffer = stencil_buffer_rc.borrow_mut();

                                        stencil_buffer.set(x, y);
                                    }

                                    if !self
                                        .options
                                        .render_pass_flags
                                        .contains(RenderPassFlag::DeferredLighting)
                                    {
                                        if let Some(forward_buffer_rc) =
                                            framebuffer.attachments.forward_ldr.as_ref()
                                        {
                                            let mut forward_buffer = forward_buffer_rc.borrow_mut();

                                            let forward_fragment_color = self
                                                .get_tone_mapped_color_from_hdr(
                                                    self.get_hdr_color_for_sample(
                                                        &shader_context,
                                                        &self.scene_resources,
                                                        &sample,
                                                    ),
                                                );

                                            forward_buffer.set(
                                                x,
                                                y,
                                                forward_fragment_color.to_u32(),
                                            );
                                        }
                                    } else {
                                        g_buffer.set(x, y, sample);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        todo!("Support framebuffers with no bound depth attachment! (i.e., always passes depth test)");
                    }
                }
            }
            None => panic!(),
        }
    }

    fn get_hdr_color_for_sample(
        &self,
        shader_context: &ShaderContext,
        scene_resources: &SceneResources,
        sample: &GeometrySample,
    ) -> Vec3 {
        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Lighting)
        {
            (self.fragment_shader)(shader_context, scene_resources, sample).to_vec3()
        } else {
            sample.albedo
        }
    }

    fn get_tone_mapped_color_from_hdr(&self, hdr_color: Vec3) -> Color {
        let mut color_tone_mapped_vec3 = hdr_color;

        if self
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Lighting)
        {
            // Exposure tone mapping

            color_tone_mapped_vec3 = hdr_color.tone_map_exposure(1.0);
        }

        // (Gamma) Transform linear space to sRGB space.

        color_tone_mapped_vec3.linear_to_srgb();

        Color::from_vec3(color_tone_mapped_vec3 * 255.0)
    }
}

fn get_vertices_in(geometry: &Geometry, face: &Face) -> [DefaultVertexIn; 3] {
    let (v0, v1, v2) = (
        geometry.vertices[face.vertices[0]],
        geometry.vertices[face.vertices[1]],
        geometry.vertices[face.vertices[2]],
    );

    let (normal0, normal1, normal2) = (
        geometry.normals[face.normals[0]],
        geometry.normals[face.normals[1]],
        geometry.normals[face.normals[2]],
    );

    let (uv0, uv1, uv2) = (
        geometry.uvs[face.uvs[0]],
        geometry.uvs[face.uvs[1]],
        geometry.uvs[face.uvs[2]],
    );

    let (tangent0, tangent1, tangent2) = (face.tangents[0], face.tangents[1], face.tangents[2]);

    let (bitangent0, bitangent1, bitangent2) =
        (face.bitangents[0], face.bitangents[1], face.bitangents[2]);

    static WHITE: Vec3 = Vec3::ones();

    let v0_in = DefaultVertexIn {
        position: v0,
        normal: normal0,
        uv: uv0,
        tangent: tangent0,
        bitangent: bitangent0,
        color: WHITE,
    };

    let v1_in = DefaultVertexIn {
        position: v1,
        normal: normal1,
        uv: uv1,
        tangent: tangent1,
        bitangent: bitangent1,
        color: WHITE,
    };

    let v2_in = DefaultVertexIn {
        position: v2,
        normal: normal2,
        uv: uv2,
        tangent: tangent2,
        bitangent: bitangent2,
        color: WHITE,
    };

    [v0_in, v1_in, v2_in]
}

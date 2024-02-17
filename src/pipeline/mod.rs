use std::sync::{RwLock, RwLockReadGuard};

use crate::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::{blend::BlendMode, Color},
    effect::Effect,
    effects::guassian_blur::GaussianBlurEffect,
    entity::Entity,
    material::{cache::MaterialCache, Material},
    matrix::Mat4,
    mesh::Face,
    shader::{
        alpha::AlphaShaderFn,
        fragment::FragmentShaderFn,
        geometry::{options::GeometryShaderOptions, sample::GeometrySample, GeometryShaderFn},
        vertex::VertexShaderFn,
        ShaderContext,
    },
    shaders::{
        default_alpha_shader::DEFAULT_ALPHA_SHADER,
        default_geometry_shader::DEFAULT_GEOMETRY_SHADER,
    },
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

use self::{gbuffer::GBuffer, options::PipelineOptions, primitive::triangle::Triangle};

use super::{
    color::{self},
    mesh::Mesh,
    vec::vec3::Vec3,
};

mod gbuffer;
pub mod options;
mod primitive;
pub mod zbuffer;

static DEFAULT_PROJECTION_Z_NEAR: f32 = 0.3;
static DEFAULT_PROJECTION_Z_FAR: f32 = 1000.0;

#[derive(Default, Debug, Copy, Clone)]
struct PipelineViewport {
    pub width: u32,
    pub width_over_2: f32,
    pub height: u32,
    pub height_over_2: f32,
}

pub struct Pipeline<'a> {
    pub options: PipelineOptions,
    framebuffer: Option<&'a RwLock<Framebuffer>>,
    viewport: PipelineViewport,
    g_buffer: Option<GBuffer>,
    bloom_buffer: Option<Buffer2D<Vec3>>,
    pub shader_context: &'a RwLock<ShaderContext>,
    vertex_shader: VertexShaderFn,
    alpha_shader: AlphaShaderFn,
    pub geometry_shader_options: GeometryShaderOptions,
    geometry_shader: GeometryShaderFn,
    fragment_shader: FragmentShaderFn,
}

impl<'a> Pipeline<'a> {
    pub fn new(
        shader_context: &'a RwLock<ShaderContext>,
        vertex_shader: VertexShaderFn,
        fragment_shader: FragmentShaderFn,
        options: PipelineOptions,
    ) -> Self {
        let alpha_shader = DEFAULT_ALPHA_SHADER;

        let geometry_shader = DEFAULT_GEOMETRY_SHADER;

        let geometry_shader_options: GeometryShaderOptions = Default::default();

        let framebuffer = None;

        let viewport: PipelineViewport = Default::default();

        return Pipeline {
            framebuffer,
            viewport,
            g_buffer: None,
            bloom_buffer: None,
            shader_context,
            vertex_shader,
            alpha_shader,
            geometry_shader,
            geometry_shader_options,
            fragment_shader,
            options,
        };
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

    pub fn bind_framebuffer(&mut self, framebuffer_option: Option<&'a RwLock<Framebuffer>>) {
        match framebuffer_option {
            Some(framebuffer_rwl) => {
                let framebuffer = framebuffer_rwl.read().unwrap();

                match framebuffer.validate() {
                    Ok(()) => {
                        self.framebuffer = framebuffer_option;

                        self.set_viewport(&framebuffer);

                        match &self.g_buffer {
                            Some(g_buffer) => {
                                if g_buffer.buffer.width != framebuffer.width
                                    || g_buffer.buffer.height != framebuffer.height
                                {
                                    // Re-allocate a G-buffer.

                                    self.g_buffer =
                                        Some(GBuffer::new(framebuffer.width, framebuffer.height));
                                }
                            }
                            None => {
                                // Re-allocate a G-buffer.

                                self.g_buffer =
                                    Some(GBuffer::new(framebuffer.width, framebuffer.height));
                            }
                        }

                        match &self.bloom_buffer {
                            Some(bloom_buffer) => {
                                if bloom_buffer.width != framebuffer.width
                                    || bloom_buffer.height != framebuffer.height
                                {
                                    // Re-allocate a bloom buffer.

                                    self.bloom_buffer = Some(Buffer2D::<Vec3>::new(
                                        framebuffer.width,
                                        framebuffer.height,
                                        None,
                                    ));
                                }
                            }
                            None => {
                                // Re-allocate a bloom buffer.

                                self.bloom_buffer = Some(Buffer2D::<Vec3>::new(
                                    framebuffer.width,
                                    framebuffer.height,
                                    None,
                                ));
                            }
                        }
                    }
                    Err(err) => {
                        panic!("Called Pipeline::bind_framebuffer() with an invalid Framebuffer! (Err: {})", err);
                    }
                }
            }
            None => {
                self.framebuffer = None;
                self.g_buffer = None;
                self.bloom_buffer = None;

                return;
            }
        }
    }

    fn set_viewport(&mut self, framebuffer: &Framebuffer) {
        self.viewport.width = framebuffer.width;
        self.viewport.width_over_2 = framebuffer.width as f32 / 2.0;
        self.viewport.height = framebuffer.height;
        self.viewport.height_over_2 = framebuffer.height as f32 / 2.0;
    }

    pub fn begin_frame(&mut self) {
        match self.framebuffer {
            Some(lock) => {
                let mut composite_framebuffer = lock.write().unwrap();

                composite_framebuffer.clear();
            }
            None => (),
        }

        if self.options.do_rasterized_geometry {
            match self.g_buffer.as_mut() {
                Some(g_buffer) => {
                    g_buffer.clear();
                }
                None => (),
            }

            match self.bloom_buffer.as_mut() {
                Some(bloom_buffer) => {
                    bloom_buffer.clear(None);
                }
                None => (),
            }
        }
    }

    pub fn end_frame(&mut self) {
        if self.options.do_rasterized_geometry {
            if self.options.do_deferred_lighting {
                self.do_deferred_lighting_pass();

                // Bloom pass over the deferred (HDR) buffer.

                if self.options.do_bloom {
                    self.do_bloom_pass();
                }
            }
        }

        // Blit deferred (HDR) framebuffer to the (final) color framebuffer.

        match self.framebuffer {
            Some(lock) => {
                let framebuffer = lock.write().unwrap();

                match (
                    framebuffer.attachments.color.as_ref(),
                    framebuffer.attachments.forward_or_deferred_hdr.as_ref(),
                ) {
                    (Some(color_buffer_lock), Some(deferred_buffer_lock)) => {
                        let (mut color_buffer, deferred_buffer) = (
                            color_buffer_lock.write().unwrap(),
                            deferred_buffer_lock.read().unwrap(),
                        );

                        for y in 0..framebuffer.height {
                            for x in 0..framebuffer.width {
                                let lit_geometry_fragment_color_tone =
                                    self.get_tone_mapped_color_from_hdr(*deferred_buffer.get(x, y));

                                color_buffer.set(x, y, lit_geometry_fragment_color_tone.to_u32());
                            }
                        }
                    }
                    _ => (),
                }

                match (
                    framebuffer.attachments.color.as_ref(),
                    framebuffer.attachments.forward_ldr.as_ref(),
                ) {
                    (Some(color_buffer_lock), Some(forward_buffer_lock)) => {
                        let (mut color_buffer, forward_buffer) = (
                            color_buffer_lock.write().unwrap(),
                            forward_buffer_lock.read().unwrap(),
                        );

                        let forward_fragments = forward_buffer.get_all();

                        // Skips pixels in our forward buffer if they weren't written to.
                        for (index, value) in forward_fragments.iter().enumerate() {
                            if Color::from_u32(*value).a > 0.0 {
                                color_buffer.set_raw(index, *value);
                            }
                        }
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }

    fn do_deferred_lighting_pass(&mut self) {
        match self.framebuffer {
            Some(lock) => {
                let mut framebuffer = lock.write().unwrap();

                match framebuffer.attachments.forward_or_deferred_hdr.as_mut() {
                    Some(deferred_buffer_lock) => {
                        let mut deferred_buffer = deferred_buffer_lock.write().unwrap();

                        // Perform deferred lighting pass.

                        let shader_context = self.shader_context.read().unwrap();

                        // Call the active fragment shader on every G-buffer sample that was
                        // written to by the rasterizer.

                        for (index, sample) in self.g_buffer.as_ref().unwrap().iter().enumerate() {
                            if sample.stencil == true {
                                let x = index as u32 % self.viewport.width;
                                let y = index as u32 / self.viewport.width;

                                let color = self.get_hdr_color_for_sample(&shader_context, &sample);

                                deferred_buffer.set(x, y, color);
                            }
                        }
                    }
                    None => (),
                }
            }
            None => (),
        }
    }

    fn do_bloom_pass(&mut self) {
        match self.framebuffer {
            Some(lock) => {
                let mut framebuffer = lock.write().unwrap();

                match framebuffer.attachments.forward_or_deferred_hdr.as_mut() {
                    Some(deferred_buffer_lock) => {
                        let mut deferred_buffer = deferred_buffer_lock.write().unwrap();

                        let mut bloom_frame = self.bloom_buffer.as_mut().unwrap();

                        for y in 0..deferred_buffer.height {
                            for x in 0..deferred_buffer.width {
                                let color_hdr = *deferred_buffer.get(x, y);

                                let perceived_brightness = if color_hdr.x >= 0.95
                                    || color_hdr.y >= 0.95
                                    || color_hdr.z >= 0.95
                                {
                                    1.0
                                } else {
                                    0.0
                                };

                                if perceived_brightness >= 1.0 {
                                    // Write this bright pixel to the initial bloom buffer.\

                                    bloom_frame.set(x, y, color_hdr);
                                }
                            }
                        }

                        // Blur the bloom buffer.

                        let bloom_effect = GaussianBlurEffect::new(6);

                        bloom_effect.apply(&mut bloom_frame);

                        // Blit the bloom buffer to our composite framebuffer.

                        deferred_buffer.blit_blended_from(
                            0,
                            0,
                            &bloom_frame,
                            Some(BlendMode::Screen),
                            Some(Vec3::ones()),
                        );
                    }
                    None => (),
                }
            }
            None => panic!(),
        }
    }

    pub fn render_entity(&mut self, entity: &Entity, material_cache: Option<&MaterialCache>) {
        self.render_entity_mesh(entity, entity.mesh, material_cache);
    }

    fn render_entity_mesh(
        &mut self,
        entity: &Entity,
        mesh: &Mesh,
        material_cache: Option<&MaterialCache>,
    ) {
        // Cull the entire entity, if possible, based on its bounds.

        if entity.mesh.normals.len() > 1 {
            let mut keep = false;

            for face in entity.bounds_mesh.faces.iter() {
                let object_vertices_in = self.get_vertices_in(&entity.bounds_mesh, &face);

                let shader_context = self.shader_context.read().unwrap();

                let projection_space_vertices: Vec<DefaultVertexOut> = object_vertices_in
                    .into_iter()
                    .map(|v_in| return (self.vertex_shader)(&shader_context, &v_in))
                    .collect();

                let mut tri: Triangle<DefaultVertexOut> = Triangle {
                    v0: projection_space_vertices[0],
                    v1: projection_space_vertices[1],
                    v2: projection_space_vertices[2],
                };

                if !self.should_cull_from_homogeneous_space(&mut tri) {
                    keep = true;
                }
            }

            if keep == false {
                return;
            }
        }

        // Otherwise, cull individual triangles.

        let world_transform = Mat4::scaling(1.0)
            * Mat4::rotation_x(entity.rotation.x)
            * Mat4::rotation_y(entity.rotation.y)
            * Mat4::rotation_z(entity.rotation.z)
            * Mat4::translation(entity.position);

        let original_world_transform: Mat4;

        {
            let mut context = self.shader_context.write().unwrap();

            original_world_transform = context.get_world_transform();

            context.set_world_transform(world_transform);
        }

        self.render_mesh(mesh, material_cache);

        // Reset the shader context's original world transform.
        {
            let mut context = self.shader_context.write().unwrap();

            context.set_world_transform(original_world_transform);
        }
    }

    fn render_mesh(&mut self, mesh: &Mesh, material_cache: Option<&MaterialCache>) {
        {
            let mut context = self.shader_context.write().unwrap();

            match &mesh.material_name {
                Some(name) => {
                    match material_cache {
                        Some(cache) => {
                            // Set the pipeline effect's active material to this
                            // mesh's material.

                            let material = cache.get(name).unwrap();
                            let material_raw_mut = &*material as *const Material;

                            context.set_active_material(Some(material_raw_mut));
                        }
                        None => (),
                    }
                }
                None => (),
            }
        }

        self.process_object_space_vertices(&mesh);

        // Reset the shader context's original active material.
        {
            let mut context = self.shader_context.write().unwrap();

            context.set_active_material(None);
        }
    }

    fn get_vertices_in(&self, mesh: &Mesh, face: &Face) -> [DefaultVertexIn; 3] {
        let v0 = mesh.vertices[face.vertices.0].clone();
        let v1 = mesh.vertices[face.vertices.1].clone();
        let v2 = mesh.vertices[face.vertices.2].clone();

        let normal0 = if face.normals.is_some() {
            mesh.normals[face.normals.unwrap().0].clone()
        } else {
            Default::default()
        };

        let normal1 = if face.normals.is_some() {
            mesh.normals[face.normals.unwrap().1].clone()
        } else {
            Default::default()
        };

        let normal2 = if face.normals.is_some() {
            mesh.normals[face.normals.unwrap().2].clone()
        } else {
            Default::default()
        };

        let uv0 = if face.uvs.is_some() {
            mesh.uvs[face.uvs.unwrap().0].clone()
        } else {
            Default::default()
        };

        let uv1 = if face.uvs.is_some() {
            mesh.uvs[face.uvs.unwrap().1].clone()
        } else {
            Default::default()
        };

        let uv2 = if face.uvs.is_some() {
            mesh.uvs[face.uvs.unwrap().2].clone()
        } else {
            Default::default()
        };

        let edge0 = v1 - v0;
        let edge1 = v2 - v0;

        let delta_uv0 = uv1 - uv0;
        let delta_uv1 = uv2 - uv0;

        let f = 1.0 / (delta_uv0.x * delta_uv1.y - delta_uv1.x * delta_uv0.y);

        let tangent = Vec3 {
            x: f * (delta_uv1.y * edge0.x - delta_uv0.y * edge1.x),
            y: f * (delta_uv1.y * edge0.y - delta_uv0.y * edge1.y),
            z: f * (delta_uv1.y * edge0.z - delta_uv0.y * edge1.z),
        };

        let bitangent = Vec3 {
            x: f * (-delta_uv1.x * edge0.x + delta_uv0.x * edge1.x),
            y: f * (-delta_uv1.x * edge0.y + delta_uv0.x * edge1.y),
            z: f * (-delta_uv1.x * edge0.z + delta_uv0.x * edge1.z),
        };

        let v0_in = DefaultVertexIn {
            position: v0,
            normal: normal0,
            tangent,
            bitangent,
            uv: uv0,
            color: color::WHITE.to_vec3() / 255.0,
        };

        let v1_in = DefaultVertexIn {
            position: v1,
            normal: normal1,
            tangent,
            bitangent,
            uv: uv1,
            color: color::WHITE.to_vec3() / 255.0,
        };

        let v2_in = DefaultVertexIn {
            position: v2,
            normal: normal2,
            tangent,
            bitangent,
            uv: uv2,
            color: color::WHITE.to_vec3() / 255.0,
        };

        [v0_in, v1_in, v2_in]
    }

    fn process_object_space_vertices(&mut self, mesh: &Mesh) {
        // Map each face to a set of 3 unique instances of DefaultVertexIn.

        let mut vertices_in: Vec<DefaultVertexIn> = vec![];

        for face_index in 0..mesh.faces.len() {
            let face = mesh.faces[face_index];

            let [v0_in, v1_in, v2_in] = self.get_vertices_in(mesh, &face);

            vertices_in.push(v0_in);
            vertices_in.push(v1_in);
            vertices_in.push(v2_in);
        }

        // Process mesh vertices from object-space to world-space.
        let projection_space_vertices: Vec<DefaultVertexOut>;

        {
            let shader_context = self.shader_context.read().unwrap();

            projection_space_vertices = vertices_in
                .into_iter()
                .map(|v_in| return (self.vertex_shader)(&shader_context, &v_in))
                .collect();
        }

        self.process_triangles(&mesh.faces, projection_space_vertices);
    }

    fn transform_to_ndc_space(&mut self, v: &mut DefaultVertexOut) {
        let w_inverse = 1.0 / v.position.w;

        *v *= w_inverse;

        v.position.x = (v.position.x + 1.0) * self.viewport.width_over_2;
        v.position.y = (-v.position.y + 1.0) * self.viewport.height_over_2;

        v.position.w = w_inverse;
    }

    fn test_and_set_z_buffer(&mut self, x: u32, y: u32, interpolant: &mut DefaultVertexOut) {
        match self.framebuffer {
            Some(lock) => {
                let framebuffer = lock.write().unwrap();

                match (
                    framebuffer.attachments.stencil.as_ref(),
                    framebuffer.attachments.depth.as_ref(),
                    framebuffer.attachments.forward_ldr.as_ref(),
                ) {
                    (
                        Some(stencil_buffer_lock),
                        Some(depth_buffer_lock),
                        Some(forward_buffer_lock),
                    ) => {
                        let mut stencil_buffer = stencil_buffer_lock.write().unwrap();
                        let mut depth_buffer = depth_buffer_lock.write().unwrap();

                        match depth_buffer.test(x, y, interpolant.position.z) {
                            Some(((x, y), non_linear_z)) => {
                                let mut linear_space_interpolant =
                                    *interpolant * (1.0 / interpolant.position.w);

                                let shader_context = self.shader_context.read().unwrap();

                                if (self.alpha_shader)(&shader_context, &linear_space_interpolant)
                                    == false
                                {
                                    return;
                                }

                                stencil_buffer.set(x, y, 1);

                                depth_buffer.set(x, y, non_linear_z);

                                match self.g_buffer.as_mut() {
                                    Some(g_buffer) => {
                                        let z = linear_space_interpolant.position.z;
                                        let near = depth_buffer.get_projection_z_near();
                                        let far = depth_buffer.get_projection_z_far();

                                        linear_space_interpolant.depth =
                                            ((z - near) / (far - near)).max(0.0).min(1.0);

                                        match (self.geometry_shader)(
                                            &shader_context,
                                            &self.geometry_shader_options,
                                            &linear_space_interpolant,
                                        ) {
                                            Some(sample) => {
                                                if self.options.do_deferred_lighting == false {
                                                    let forward_fragment_color = self
                                                        .get_tone_mapped_color_from_hdr(
                                                            self.get_hdr_color_for_sample(
                                                                &shader_context,
                                                                &sample,
                                                            ),
                                                        );

                                                    let mut forward_buffer =
                                                        forward_buffer_lock.write().unwrap();

                                                    forward_buffer.set(
                                                        x,
                                                        y,
                                                        forward_fragment_color.to_u32(),
                                                    );
                                                } else {
                                                    g_buffer.set(x, y, sample);
                                                }
                                            }
                                            None => (),
                                        }
                                    }
                                    None => (),
                                }
                            }
                            None => {}
                        }
                    }
                    _ => {
                        todo!("Support framebuffers with no bound depth attachment or no bound forward (LDR) attachment!");
                    }
                }
            }
            None => panic!(),
        }
    }

    fn get_hdr_color_for_sample(
        &self,
        shader_context: &RwLockReadGuard<'_, ShaderContext>,
        sample: &GeometrySample,
    ) -> Vec3 {
        if self.options.do_lighting {
            (self.fragment_shader)(&shader_context, &sample).to_vec3()
        } else {
            sample.diffuse
        }
    }

    fn get_tone_mapped_color_from_hdr(&self, hdr_color: Vec3) -> Color {
        let mut color_tone_mapped_vec3 = hdr_color;

        if self.options.do_lighting {
            // Exposure tone mapping

            static EXPOSURE: f32 = 1.0;

            color_tone_mapped_vec3 = Vec3::ones()
                - Vec3 {
                    x: (-hdr_color.x * EXPOSURE).exp(),
                    y: (-hdr_color.y * EXPOSURE).exp(),
                    z: (-hdr_color.z * EXPOSURE).exp(),
                };
        }

        // (Gamma) Transform linear space to sRGB space.

        color_tone_mapped_vec3.linear_to_srgb();

        Color::from_vec3(color_tone_mapped_vec3 * 255.0)
    }
}

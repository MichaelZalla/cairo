use std::sync::RwLock;

use crate::{
    buffer::Buffer2D,
    color::{blend::BlendMode, Color},
    effect::Effect,
    effects::guassian_blur::GaussianBlurEffect,
    entity::Entity,
    material::{cache::MaterialCache, Material},
    matrix::Mat4,
    mesh::Face,
    shader::{
        alpha::AlphaShaderFn, fragment::FragmentShaderFn, geometry::GeometryShader,
        vertex::VertexShaderFn, ShaderContext,
    },
    shaders::{
        default_alpha_shader::DefaultAlphaShader, default_geometry_shader::DefaultGeometryShader,
    },
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

use self::{
    gbuffer::GBuffer,
    options::{PipelineFaceCullingReject, PipelineFaceCullingWindingOrder, PipelineOptions},
    zbuffer::ZBuffer,
};

use super::{
    color::{self},
    mesh::Mesh,
    vec::{vec3::Vec3, vec4::Vec4},
};

mod gbuffer;
pub mod options;
mod primitive;
mod zbuffer;

static DEFAULT_PROJECTION_Z_NEAR: f32 = 0.3;
static DEFAULT_PROJECTION_Z_FAR: f32 = 1000.0;

#[derive(Default, Debug, Copy, Clone)]
struct Triangle<T> {
    v0: T,
    v1: T,
    v2: T,
}

#[derive(Default, Debug, Copy, Clone)]
struct PipelineViewport {
    pub width: u32,
    pub width_over_2: f32,
    pub height: u32,
    pub height_over_2: f32,
}

pub struct Pipeline<'a, G = DefaultGeometryShader<'a>>
where
    G: GeometryShader<'a>,
{
    pub options: PipelineOptions,
    forward_framebuffer: Option<Buffer2D>,
    deferred_framebuffer: Option<Buffer2D<Vec3>>,
    composite_framebuffer: Option<&'a RwLock<Buffer2D>>,
    keying_color: u32,
    viewport: PipelineViewport,
    z_buffer: Option<ZBuffer>,
    g_buffer: Option<GBuffer>,
    bloom_buffer: Option<Buffer2D<Vec3>>,
    pub shader_context: &'a RwLock<ShaderContext>,
    vertex_shader: VertexShaderFn,
    alpha_shader: AlphaShaderFn,
    pub geometry_shader: G,
    fragment_shader: FragmentShaderFn,
}

impl<'a, G> Pipeline<'a, G>
where
    G: GeometryShader<'a>,
{
    pub fn new(
        shader_context: &'a RwLock<ShaderContext>,
        vertex_shader: VertexShaderFn,
        geometry_shader: G,
        fragment_shader: FragmentShaderFn,
        options: PipelineOptions,
    ) -> Self {
        let alpha_shader = DefaultAlphaShader;

        let forward_framebuffer = None;

        let deferred_framebuffer = None;

        let composite_framebuffer = None;

        let keying_color = color::BLACK.to_u32();

        let viewport: PipelineViewport = Default::default();

        return Pipeline {
            forward_framebuffer,
            deferred_framebuffer,
            composite_framebuffer,
            keying_color,
            viewport,
            z_buffer: None,
            g_buffer: None,
            bloom_buffer: None,
            shader_context,
            vertex_shader,
            alpha_shader,
            geometry_shader,
            fragment_shader,
            options,
        };
    }

    pub fn set_projection_z_near(&mut self, depth: f32) {
        match self.z_buffer.as_mut() {
            Some(z_buffer) => {
                z_buffer.set_projection_z_near(depth);
            }
            None => {
                panic!(
                    "Called Pipeline::set_projection_z_near() on pipeline with no bound Z-buffer!"
                );
            }
        }
    }

    pub fn set_projection_z_far(&mut self, depth: f32) {
        match self.z_buffer.as_mut() {
            Some(z_buffer) => {
                z_buffer.set_projection_z_far(depth);
            }
            None => {
                panic!(
                    "Called Pipeline::set_projection_z_far() on pipeline with no bound Z-buffer!"
                );
            }
        }
    }

    pub fn set_vertex_shader(&mut self, shader: VertexShaderFn) {
        self.vertex_shader = shader;
    }

    pub fn set_fragment_shader(&mut self, shader: FragmentShaderFn) {
        self.fragment_shader = shader;
    }

    pub fn bind_framebuffer(&mut self, framebuffer_option: Option<&'a RwLock<Buffer2D>>) {
        self.composite_framebuffer = framebuffer_option;

        match framebuffer_option {
            Some(framebuffer_rwl) => {
                let framebuffer = framebuffer_rwl.read().unwrap();

                self.set_viewport(&framebuffer);

                match &self.forward_framebuffer {
                    Some(buffer) => {
                        if buffer.width != framebuffer.width || buffer.height != framebuffer.height
                        {
                            // Re-allocate a forward framebuffer.

                            self.forward_framebuffer =
                                Some(Buffer2D::new(framebuffer.width, framebuffer.height, None))
                        }
                    }
                    None => {
                        // Re-allocate a forward framebuffer.

                        self.forward_framebuffer =
                            Some(Buffer2D::new(framebuffer.width, framebuffer.height, None))
                    }
                }

                match &self.deferred_framebuffer {
                    Some(buffer) => {
                        if buffer.width != framebuffer.width || buffer.height != framebuffer.height
                        {
                            // Re-allocate a deferred framebuffer.

                            self.deferred_framebuffer =
                                Some(Buffer2D::new(framebuffer.width, framebuffer.height, None));
                        }
                    }
                    None => {
                        // Re-allocate a deferred framebuffer.

                        self.deferred_framebuffer =
                            Some(Buffer2D::new(framebuffer.width, framebuffer.height, None));
                    }
                }

                match &self.z_buffer {
                    Some(z_buffer) => {
                        if z_buffer.buffer.width != framebuffer.width
                            || z_buffer.buffer.height != framebuffer.height
                        {
                            // Re-allocate a Z-buffer.

                            self.z_buffer = Some(ZBuffer::new(
                                framebuffer.width,
                                framebuffer.height,
                                z_buffer.get_projection_z_near(),
                                z_buffer.get_projection_z_far(),
                            ));
                        }
                    }
                    None => {
                        // Re-allocate a Z-buffer.

                        self.z_buffer = Some(ZBuffer::new(
                            framebuffer.width,
                            framebuffer.height,
                            DEFAULT_PROJECTION_Z_NEAR,
                            DEFAULT_PROJECTION_Z_FAR,
                        ));
                    }
                }

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

                        self.g_buffer = Some(GBuffer::new(framebuffer.width, framebuffer.height));
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
            None => {
                self.forward_framebuffer = None;
                self.deferred_framebuffer = None;
                self.z_buffer = None;
                self.g_buffer = None;
                self.bloom_buffer = None;

                return;
            }
        }
    }

    fn set_viewport(&mut self, framebuffer: &Buffer2D) {
        self.viewport.width = framebuffer.width;
        self.viewport.width_over_2 = framebuffer.width as f32 / 2.0;
        self.viewport.height = framebuffer.height;
        self.viewport.height_over_2 = framebuffer.height as f32 / 2.0;
    }

    pub fn begin_frame(&mut self, fill_value_option: Option<Color>) {
        let fill_value = match fill_value_option {
            Some(color) => color.to_u32(),
            None => color::BLACK.to_u32(),
        };

        self.set_keying_color(fill_value);

        match self.forward_framebuffer.as_mut() {
            Some(forward_framebuffer) => {
                forward_framebuffer.clear(Some(fill_value));
            }
            None => (),
        }

        match self.composite_framebuffer {
            Some(lock) => {
                let mut composite_framebuffer = lock.write().unwrap();

                composite_framebuffer.clear(Some(fill_value));
            }
            None => (),
        }

        if self.options.do_rasterized_geometry {
            match self.deferred_framebuffer.as_mut() {
                Some(deferred_framebuffer) => {
                    let fill_value_vec3 = Color::from_u32(fill_value).to_vec3();

                    deferred_framebuffer.clear(Some(fill_value_vec3));
                }
                None => (),
            }

            match self.z_buffer.as_mut() {
                Some(z_buffer) => {
                    z_buffer.clear();
                }
                None => (),
            }

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
            // Perform deferred lighting pass.

            let shader_context = self.shader_context.read().unwrap();

            // Call the active fragment shader on every G-buffer sample that was
            // written to by the rasterizer.

            for (index, sample) in self.g_buffer.as_ref().unwrap().iter().enumerate() {
                if sample.stencil == true {
                    let x = index as u32 % self.viewport.width;
                    let y = index as u32 / self.viewport.width;

                    let color = if self.options.do_lighting {
                        (self.fragment_shader)(&shader_context, &sample)
                    } else {
                        Color::from_vec3(sample.diffuse)
                    };

                    self.deferred_framebuffer
                        .as_mut()
                        .unwrap()
                        .set(x, y, color.to_vec3());
                }
            }
        }

        // Bloom pass over the deferred (HDR) buffer.

        if self.options.do_bloom {
            self.do_bloom_pass();
        }

        // Compose deferred and forward rendering frames together.

        let mut composite_framebuffer = match self.composite_framebuffer {
            Some(composite_framebuffer) => composite_framebuffer.write().unwrap(),
            None => {
                panic!("Called Pipeline::end_frame() with no bound composite framebuffer!");
            }
        };

        if self.options.do_rasterized_geometry {
            let deferred_frame = self.deferred_framebuffer.as_ref().unwrap();

            for y in 0..composite_framebuffer.height {
                for x in 0..composite_framebuffer.width {
                    let color_hdr_vec3 = *deferred_frame.get(x, y);

                    let color_tone_mapped = self.get_tone_mapped_color_from_hdr(color_hdr_vec3);

                    composite_framebuffer.set(x, y, color_tone_mapped.to_u32());
                }
            }
        }

        let forward_frame = self.forward_framebuffer.as_ref().unwrap().get_all();

        // Skips pixels in our forward buffer if they weren't written to.
        for (index, value) in forward_frame.iter().enumerate() {
            if *value != self.keying_color {
                composite_framebuffer.set_raw(index, *value);
            }
        }
    }

    fn do_bloom_pass(&mut self) {
        let deferred_frame = self.deferred_framebuffer.as_mut().unwrap();

        let mut bloom_frame = self.bloom_buffer.as_mut().unwrap();

        for y in 0..deferred_frame.height {
            for x in 0..deferred_frame.width {
                let color_hdr = *deferred_frame.get(x, y);

                let perceived_brightness =
                    if color_hdr.x >= 0.95 || color_hdr.y >= 0.95 || color_hdr.z >= 0.95 {
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

        deferred_frame.blit_blended_from(
            0,
            0,
            &bloom_frame,
            Some(BlendMode::Screen),
            Some(Vec3::ones()),
        );
    }

    pub fn set_keying_color(&mut self, color: u32) {
        self.keying_color = color;
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

        let v0_in = DefaultVertexIn {
            position: v0,
            normal: normal0,
            uv: uv0,
            color: color::WHITE.to_vec3() / 255.0,
        };

        let v1_in = DefaultVertexIn {
            position: v1,
            normal: normal1,
            uv: uv1,
            color: color::WHITE.to_vec3() / 255.0,
        };

        let v2_in = DefaultVertexIn {
            position: v2,
            normal: normal2,
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

    fn process_triangles(
        &mut self,
        faces: &Vec<Face>,
        projection_space_vertices: Vec<DefaultVertexOut>,
    ) {
        let mut triangles: Vec<Triangle<DefaultVertexOut>> = vec![];

        for face_index in 0..faces.len() {
            // Cull backfaces

            let mut v0 = projection_space_vertices[face_index * 3];
            let mut v1 = projection_space_vertices[face_index * 3 + 1];
            let mut v2 = projection_space_vertices[face_index * 3 + 2];

            match self.options.face_culling_strategy.window_order {
                PipelineFaceCullingWindingOrder::Clockwise => {
                    (v0, v1, v2) = (v2, v1, v0);
                }
                PipelineFaceCullingWindingOrder::CounterClockwise => {
                    // Use default (counter-clockwise) ordering.
                }
            }

            match self.options.face_culling_strategy.reject {
                PipelineFaceCullingReject::None => {
                    // Render all faces.
                }
                PipelineFaceCullingReject::Backfaces => {
                    // Reject backfaces.

                    if self.is_backface(v0.position, v1.position, v2.position) {
                        continue;
                    }
                }
                PipelineFaceCullingReject::Frontfaces => {
                    // Reject frontfaces.

                    if !self.is_backface(v0.position, v1.position, v2.position) {
                        continue;
                    }
                }
            }

            triangles.push(Triangle { v0, v1, v2 });
        }

        for triangle in triangles.as_mut_slice() {
            self.process_triangle(triangle);
        }
    }

    fn is_backface(&mut self, v0: Vec4, v1: Vec4, v2: Vec4) -> bool {
        let vertices = [
            Vec3 {
                x: v0.x,
                y: v0.y,
                z: v0.z,
            },
            Vec3 {
                x: v1.x,
                y: v1.y,
                z: v1.z,
            },
            Vec3 {
                x: v2.x,
                y: v2.y,
                z: v2.z,
            },
        ];

        // Computes a hard surface normal for the face (ignores smooth normals);

        let vertex_normal = (vertices[1] - vertices[0])
            .cross(vertices[2] - vertices[0])
            .as_normal();

        let projected_origin = Vec4::new(Default::default(), 1.0)
            * self.shader_context.read().unwrap().get_projection();

        let dot_product = vertex_normal.dot(
            vertices[0].as_normal()
                - Vec3 {
                    x: projected_origin.x,
                    y: projected_origin.y,
                    z: projected_origin.z,
                },
        );

        if dot_product > 0.0 {
            return true;
        }

        return false;
    }

    fn should_cull_from_homogeneous_space(
        &mut self,
        triangle: &mut Triangle<DefaultVertexOut>,
    ) -> bool {
        if triangle.v0.position.x > triangle.v0.position.w
            && triangle.v1.position.x > triangle.v1.position.w
            && triangle.v2.position.x > triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.x < -triangle.v0.position.w
            && triangle.v1.position.x < -triangle.v1.position.w
            && triangle.v2.position.x < -triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.y > triangle.v0.position.w
            && triangle.v1.position.y > triangle.v1.position.w
            && triangle.v2.position.y > triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.y < -triangle.v0.position.w
            && triangle.v1.position.y < -triangle.v1.position.w
            && triangle.v2.position.y < -triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.z > triangle.v0.position.w
            && triangle.v1.position.z > triangle.v1.position.w
            && triangle.v2.position.z > triangle.v2.position.w
        {
            return true;
        }

        if triangle.v0.position.z < 0.0
            && triangle.v1.position.z < 0.0
            && triangle.v2.position.z < 0.0
        {
            return true;
        }

        return false;
    }

    fn clip1(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let a_alpha = -(v0.position.z) / (v1.position.z - v0.position.z);
        let b_alpha = -(v0.position.z) / (v2.position.z - v0.position.z);

        let a_prime = DefaultVertexOut::interpolate(v0, v1, a_alpha);
        let b_prime = DefaultVertexOut::interpolate(v0, v2, b_alpha);

        let mut triangle1 = Triangle {
            v0: a_prime,
            v1,
            v2,
        };

        let mut triangle2 = Triangle {
            v0: b_prime,
            v1: a_prime,
            v2,
        };

        self.post_process_triangle_vertices(&mut triangle1);
        self.post_process_triangle_vertices(&mut triangle2);
    }

    fn clip2(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let a_alpha = -(v0.position.z) / (v2.position.z - v0.position.z);
        let b_alpha = -(v1.position.z) / (v2.position.z - v1.position.z);

        let a_prime = DefaultVertexOut::interpolate(v0, v2, a_alpha);
        let b_prime = DefaultVertexOut::interpolate(v1, v2, b_alpha);

        let mut triangle = Triangle {
            v0: a_prime,
            v1: b_prime,
            v2,
        };

        self.post_process_triangle_vertices(&mut triangle);
    }

    fn process_triangle(&mut self, triangle: &mut Triangle<DefaultVertexOut>) {
        // @TODO(mzalla) Geometry shader?

        if self.should_cull_from_homogeneous_space(triangle) {
            return;
        }

        // Clip triangles that intersect the front of our view frustum

        if triangle.v0.position.z < 0.0 {
            if triangle.v1.position.z < 0.0 {
                // Clip 2 (0 and 1)
                self.clip2(triangle.v0, triangle.v1, triangle.v2);
            } else if triangle.v2.position.z < 0.0 {
                // Clip 2 (0 and 2)
                self.clip1(triangle.v0, triangle.v2, triangle.v1);
            } else {
                // Clip 1 (0)
                self.clip1(triangle.v0, triangle.v1, triangle.v2);
            }
        } else if triangle.v1.position.z < 0.0 {
            if triangle.v2.position.z < 0.0 {
                // Clip 2
                self.clip2(triangle.v1, triangle.v2, triangle.v0);
            } else {
                // Clip 1
                self.clip1(triangle.v1, triangle.v0, triangle.v2);
            }
        } else if triangle.v2.position.z < 0.0 {
            // Clip 1
            self.clip1(triangle.v2, triangle.v0, triangle.v1);
        } else {
            self.post_process_triangle_vertices(triangle);
        }
    }

    fn transform_to_ndc_space(&mut self, v: &mut DefaultVertexOut) {
        let w_inverse = 1.0 / v.position.w;

        *v *= w_inverse;

        v.position.x = (v.position.x + 1.0) * self.viewport.width_over_2;
        v.position.y = (-v.position.y + 1.0) * self.viewport.height_over_2;

        v.position.w = w_inverse;
    }

    fn post_process_triangle_vertices(&mut self, triangle: &mut Triangle<DefaultVertexOut>) {
        // World-space to screen-space (NDC) transform

        let projection_space_vertices = [triangle.v0, triangle.v1, triangle.v2];

        let mut ndc_space_vertices = projection_space_vertices.clone();

        self.transform_to_ndc_space(&mut ndc_space_vertices[0]);
        self.transform_to_ndc_space(&mut ndc_space_vertices[1]);
        self.transform_to_ndc_space(&mut ndc_space_vertices[2]);

        // Interpolate entire vertex (all attributes) when drawing (scanline
        // interpolant)

        if self.options.do_rasterized_geometry {
            self.triangle_fill(
                ndc_space_vertices[0],
                ndc_space_vertices[1],
                ndc_space_vertices[2],
            );
        }

        if self.options.do_wireframe {
            for i in 0..3 {
                self.render_line(
                    projection_space_vertices[i].world_pos,
                    projection_space_vertices[if i == 2 { 0 } else { i + 1 }].world_pos,
                    color::WHITE,
                );
            }
        }

        if self.options.do_visualize_normals {
            for i in 0..3 {
                self.render_line(
                    projection_space_vertices[i].world_pos,
                    projection_space_vertices[i].world_pos
                        + projection_space_vertices[i].normal.to_vec3(),
                    color::BLUE,
                );
            }
        }
    }

    fn test_and_set_z_buffer(&mut self, x: u32, y: u32, interpolant: &mut DefaultVertexOut) {
        let z_buffer = self.z_buffer.as_mut().unwrap();

        match z_buffer.test(x, y, interpolant.position.z) {
            Some(((x, y), non_linear_z)) => {
                let mut linear_space_interpolant = *interpolant * (1.0 / interpolant.position.w);

                let context = self.shader_context.read().unwrap();

                if (self.alpha_shader)(&context, &linear_space_interpolant) == false {
                    return;
                }

                z_buffer.set(x, y, non_linear_z);

                match self.g_buffer.as_mut() {
                    Some(g_buffer) => {
                        let z = linear_space_interpolant.position.z;
                        let near = z_buffer.get_projection_z_near();
                        let far = z_buffer.get_projection_z_far();

                        linear_space_interpolant.depth =
                            ((z - near) / (far - near)).max(0.0).min(1.0);

                        g_buffer.set(x, y, self.geometry_shader.call(&linear_space_interpolant));
                    }
                    None => (),
                }
            }
            None => {}
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

        color_tone_mapped_vec3 = Vec3 {
            x: color_tone_mapped_vec3.x.sqrt(),
            y: color_tone_mapped_vec3.y.sqrt(),
            z: color_tone_mapped_vec3.z.sqrt(),
        };

        Color::from_vec3(color_tone_mapped_vec3 * 255.0)
    }

    fn flat_top_triangle_fill(
        &mut self,
        top_left: DefaultVertexOut,
        top_right: DefaultVertexOut,
        bottom: DefaultVertexOut,
    ) {
        let delta_y = bottom.position.y - top_left.position.y;

        // Calculate the change (step) for left and right sides, as we
        // rasterize downwards with each scanline.
        let top_left_step = (bottom - top_left) / delta_y;
        let top_right_step = (bottom - top_right) / delta_y;

        // Create the right edge interpolant.
        let mut right_edge_interpolant = top_right;

        self.flat_triangle_fill(
            &top_left,
            // &top_right,
            &bottom,
            &top_left_step,
            &top_right_step,
            &mut right_edge_interpolant,
        );
    }

    fn flat_bottom_triangle_fill(
        &mut self,
        top: DefaultVertexOut,
        bottom_left: DefaultVertexOut,
        bottom_right: DefaultVertexOut,
    ) {
        let delta_y = bottom_right.position.y - top.position.y;

        // Calculate the change (step) for both left and right sides, as we
        // rasterize downwards with each scanline.
        let bottom_left_step = (bottom_left - top) / delta_y;
        let bottom_right_step = (bottom_right - top) / delta_y;

        // Create the right edge interpolant.
        let mut right_edge_interpolant = top;

        self.flat_triangle_fill(
            &top,
            // &bottom_left,
            &bottom_right,
            &bottom_left_step,
            &bottom_right_step,
            &mut right_edge_interpolant,
        );
    }

    fn flat_triangle_fill(
        &mut self,
        it0: &DefaultVertexOut,
        // it1: &DefaultVertexOut,
        it2: &DefaultVertexOut,
        left_step: &DefaultVertexOut,
        right_step: &DefaultVertexOut,
        right_edge_interpolant: &mut DefaultVertexOut,
    ) {
        // it0 will always be a top vertex.
        // it1 is either a top or a bottom vertex.
        // it2 will always be a bottom vertex.

        // Case 1. Flat-top triangle:
        //  - Left-edge interpolant begins at top-left vertex.
        //  - Right-edge interpolant begins at top-right vertex.

        // Case 2. Flat-bottom triangle:
        //  - Left-edge and right-edge interpolants both begin at top vertex.

        // Left edge is always it0
        let mut left_edge_interpolant = it0.clone();

        // Calculate our start and end Y (end here is non-inclusive), such that
        // they are non-fractional screen coordinates.
        let y_start: u32 = u32::max((it0.position.y - 0.5).ceil() as u32, 0);
        let y_end: u32 = u32::min(
            (it2.position.y - 0.5).ceil() as u32,
            self.viewport.height - 1,
        );

        // Adjust both interpolants to account for us snapping y-start and y-end
        // to their nearest whole pixel coordinates.
        left_edge_interpolant += *left_step * (y_start as f32 + 0.5 - it0.position.y);
        *right_edge_interpolant += *right_step * (y_start as f32 + 0.5 - it0.position.y);

        // Rasterization loop
        for y in y_start..y_end {
            // Calculate our start and end X (end here is non-inclusive), such
            // that they are non-fractional screen coordinates.
            let x_start = u32::max((left_edge_interpolant.position.x - 0.5).ceil() as u32, 0);
            let x_end = u32::min(
                (right_edge_interpolant.position.x - 0.5).ceil() as u32,
                self.viewport.width - 1,
            );

            // Create an interpolant that we can move across our horizontal
            // scanline.
            let mut line_interpolant = left_edge_interpolant.clone();

            // Calculate the width of our scanline, for this Y position.
            let dx = right_edge_interpolant.position.x - left_edge_interpolant.position.x;

            // Calculate the change (step) for our horizontal interpolant, based
            // on the width of our scanline.
            let line_interpolant_step = (*right_edge_interpolant - line_interpolant) / dx;

            // Prestep our scanline interpolant to account for us snapping
            // x-start and x-end to their nearest whole pixel coordinates.
            line_interpolant +=
                line_interpolant_step * ((x_start as f32) + 0.5 - left_edge_interpolant.position.x);

            for x in x_start..x_end {
                self.test_and_set_z_buffer(x, y, &mut line_interpolant);

                line_interpolant += line_interpolant_step;
            }

            left_edge_interpolant += *left_step;
            *right_edge_interpolant += *right_step;
        }
    }

    fn triangle_fill(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let mut tri = vec![v0, v1, v2];

        // Sorts points by y-value (highest-to-lowest)

        if tri[1].position.y < tri[0].position.y {
            tri.swap(0, 1);
        }
        if tri[2].position.y < tri[1].position.y {
            tri.swap(1, 2);
        }
        if tri[1].position.y < tri[0].position.y {
            tri.swap(0, 1);
        }

        if tri[0].position.y == tri[1].position.y {
            // Flat-top (horizontal line is tri[0]-to-tri[1]);

            // tri[2] must sit below tri[0] and tri[1]; tri[0] and tri[1] cannot
            // have the same x-value; therefore, sort tri[0] and tri[1] by x-value;

            if tri[1].position.x < tri[0].position.x {
                tri.swap(0, 1);
            }

            self.flat_top_triangle_fill(tri[0], tri[1], tri[2]);
        } else if tri[1].position.y == tri[2].position.y {
            // Flat-bottom (horizontal line is tri[1]-to-tri[2]);

            // tri[0] must sit above tri[1] and tri[2]; tri[1] and tri[2] cannot
            // have the same x-value; therefore, sort tri[1] and tri[2] by x-value;

            if tri[2].position.x < tri[1].position.x {
                tri.swap(1, 2);
            }

            self.flat_bottom_triangle_fill(tri[0], tri[1], tri[2]);
        } else {
            // Find splitting vertex

            let alpha_split =
                (tri[1].position.y - tri[0].position.y) / (tri[2].position.y - tri[0].position.y);

            let split_vertex = DefaultVertexOut::interpolate(tri[0], tri[2], alpha_split);

            if tri[1].position.x < split_vertex.position.x {
                // Major right

                // tri[0] must sit above tri[1] and split_point; tri[1] and
                // split_point cannot have the same x-value; therefore, sort tri[1]
                // and split_point by x-value;

                self.flat_bottom_triangle_fill(tri[0], tri[1], split_vertex);

                self.flat_top_triangle_fill(tri[1], split_vertex, tri[2]);
            } else {
                // Major left

                self.flat_bottom_triangle_fill(tri[0], split_vertex, tri[1]);

                self.flat_top_triangle_fill(split_vertex, tri[1], tri[2]);
            }
        }
    }
}

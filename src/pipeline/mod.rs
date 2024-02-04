use std::sync::RwLock;

use crate::{
    buffer::Buffer2D,
    color::Color,
    entity::Entity,
    material::{cache::MaterialCache, Material},
    matrix::Mat4,
    mesh::Face,
    shader::{
        alpha::AlphaShader, fragment::FragmentShader, geometry::GeometryShader,
        vertex::VertexShader, ShaderContext,
    },
    shaders::{
        default_alpha_shader::DefaultAlphaShader, default_fragment_shader::DefaultFragmentShader,
        default_geometry_shader::DefaultGeometryShader, default_vertex_shader::DefaultVertexShader,
    },
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

use self::{gbuffer::GBuffer, options::PipelineOptions, zbuffer::ZBuffer};

use super::{
    color::{self},
    graphics::Graphics,
    mesh::Mesh,
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
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

pub struct Pipeline<
    'a,
    F = DefaultFragmentShader<'a>,
    V = DefaultVertexShader<'a>,
    A = DefaultAlphaShader<'a>,
    G = DefaultGeometryShader<'a>,
> where
    F: FragmentShader<'a>,
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
    pub options: PipelineOptions,
    forward_framebuffer: Option<Buffer2D>,
    deferred_framebuffer: Option<Buffer2D>,
    composite_framebuffer: Option<&'a RwLock<Buffer2D>>,
    viewport: PipelineViewport,
    projection_z_near: f32,
    projection_z_far: f32,
    z_buffer: Option<ZBuffer>,
    g_buffer: Option<GBuffer>,
    pub shader_context: &'a RwLock<ShaderContext>,
    vertex_shader: V,
    alpha_shader: A,
    pub geometry_shader: G,
    fragment_shader: F,
}

impl<'a, F, V, A, G> Pipeline<'a, F, V, A, G>
where
    F: FragmentShader<'a>,
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
    pub fn new(
        shader_context: &'a RwLock<ShaderContext>,
        vertex_shader: V,
        geometry_shader: G,
        fragment_shader: F,
        options: PipelineOptions,
    ) -> Self {
        let alpha_shader = AlphaShader::new(shader_context);

        let forward_framebuffer = None;

        let deferred_framebuffer = None;

        let composite_framebuffer = None;

        let viewport: PipelineViewport = Default::default();

        let z_buffer = None;

        let g_buffer = None;

        return Pipeline {
            forward_framebuffer,
            deferred_framebuffer,
            composite_framebuffer,
            viewport,
            projection_z_near: DEFAULT_PROJECTION_Z_NEAR,
            projection_z_far: DEFAULT_PROJECTION_Z_FAR,
            z_buffer,
            g_buffer,
            shader_context,
            vertex_shader,
            alpha_shader,
            geometry_shader,
            fragment_shader,
            options,
        };
    }

    pub fn set_projection_z_near(&mut self, depth: f32) {
        self.projection_z_near = depth;

        match self.z_buffer.as_mut() {
            Some(z_buffer) => {
                z_buffer.set_projection_z_near(depth);
            }
            None => (),
        }
    }

    pub fn set_projection_z_far(&mut self, depth: f32) {
        self.projection_z_far = depth;

        match self.z_buffer.as_mut() {
            Some(z_buffer) => {
                z_buffer.set_projection_z_far(depth);
            }
            None => (),
        }
    }

    pub fn bind_framebuffer(&mut self, framebuffer_option: Option<&'a RwLock<Buffer2D>>) {
        self.composite_framebuffer = framebuffer_option;

        match framebuffer_option {
            Some(framebuffer_rwl) => {
                let framebuffer = framebuffer_rwl.read().unwrap();

                self.set_viewport(&framebuffer);

                let black = color::BLACK.to_u32();

                match &self.forward_framebuffer {
                    Some(buffer) => {
                        if buffer.width != framebuffer.width || buffer.height != framebuffer.height
                        {
                            // Re-allocate a forward framebuffer.

                            self.forward_framebuffer = Some(Buffer2D::new(
                                framebuffer.width,
                                framebuffer.height,
                                Some(black),
                            ))
                        }
                    }
                    None => {
                        // Re-allocate a forward framebuffer.

                        self.forward_framebuffer = Some(Buffer2D::new(
                            framebuffer.width,
                            framebuffer.height,
                            Some(black),
                        ))
                    }
                }

                match &self.deferred_framebuffer {
                    Some(buffer) => {
                        if buffer.width != framebuffer.width || buffer.height != framebuffer.height
                        {
                            // Re-allocate a deferred framebuffer.

                            self.deferred_framebuffer = Some(Buffer2D::new(
                                framebuffer.width,
                                framebuffer.height,
                                Some(black),
                            ));
                        }
                    }
                    None => {
                        // Re-allocate a deferred framebuffer.

                        self.deferred_framebuffer = Some(Buffer2D::new(
                            framebuffer.width,
                            framebuffer.height,
                            Some(black),
                        ));
                    }
                }

                match &self.z_buffer {
                    Some(zbuffer) => {
                        if zbuffer.buffer.width != framebuffer.width
                            || zbuffer.buffer.height != framebuffer.height
                        {
                            // Re-allocate a Z-buffer.

                            self.z_buffer = Some(ZBuffer::new(
                                framebuffer.width,
                                framebuffer.height,
                                self.projection_z_near,
                                self.projection_z_far,
                            ));
                        }
                    }
                    None => {
                        // Re-allocate a Z-buffer.

                        self.z_buffer = Some(ZBuffer::new(
                            framebuffer.width,
                            framebuffer.height,
                            self.projection_z_near,
                            self.projection_z_far,
                        ));
                    }
                }

                match &self.g_buffer {
                    Some(gbuffer) => {
                        if gbuffer.buffer.width != framebuffer.width
                            || gbuffer.buffer.height != framebuffer.height
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
            }
            None => {
                self.forward_framebuffer = None;
                self.deferred_framebuffer = None;
                self.z_buffer = None;
                self.g_buffer = None;

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

    pub fn begin_frame(&mut self) {
        let fill_value = color::BLACK.to_u32();

        self.forward_framebuffer
            .as_mut()
            .unwrap()
            .clear(Some(fill_value));

        self.composite_framebuffer
            .unwrap()
            .write()
            .unwrap()
            .clear(Some(fill_value));

        if self.options.show_rasterized_geometry {
            self.deferred_framebuffer
                .as_mut()
                .unwrap()
                .clear(Some(fill_value));

            self.z_buffer.as_mut().unwrap().clear();

            self.g_buffer.as_mut().unwrap().clear();
        }
    }

    pub fn end_frame(&mut self) {
        if self.options.show_rasterized_geometry {
            // Perform deferred lighting pass.

            // Call the active fragment shader on every G-buffer sample that was
            // written to by the rasterizer.

            for (index, sample) in self.g_buffer.as_ref().unwrap().iter().enumerate() {
                if sample.stencil == true {
                    let x = index as u32 % self.viewport.width;
                    let y = index as u32 / self.viewport.width;

                    let color = if self.options.show_lighting {
                        self.fragment_shader.call(&sample)
                    } else {
                        Color::from_vec3(sample.diffuse * 255.0)
                    };

                    self.deferred_framebuffer
                        .as_mut()
                        .unwrap()
                        .set(x, y, color.to_u32());
                }
            }
        }

        // Compose deferred and forward rendering frames together.

        let mut composite_framebuffer = match self.composite_framebuffer {
            Some(composite_framebuffer) => composite_framebuffer.write().unwrap(),
            None => {
                panic!("Called Pipeline::end_frame() with no bound composite framebuffer!");
            }
        };

        if self.options.show_rasterized_geometry {
            composite_framebuffer.blit_from(0, 0, self.deferred_framebuffer.as_ref().unwrap());
        }

        let forward_frame = self.forward_framebuffer.as_ref().unwrap().get_all();

        // Skips pixels in our forward buffer if they weren't written to.
        let keying_color = color::BLACK.to_u32();

        for (index, value) in forward_frame.iter().enumerate() {
            if *value != keying_color {
                composite_framebuffer.set_raw(index, *value);
            }
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

                let world_vertices: Vec<DefaultVertexOut> = object_vertices_in
                    .into_iter()
                    .map(|v_in| return self.vertex_shader.call(&v_in))
                    .collect();

                let mut tri: Triangle<DefaultVertexOut> = Triangle {
                    v0: world_vertices[0],
                    v1: world_vertices[1],
                    v2: world_vertices[2],
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
                            let mat = cache.get(name).unwrap();
                            let mat_raw_mut = &*mat as *const Material;

                            context.set_active_material(Some(mat_raw_mut));
                        }
                        None => (),
                    }
                }
                None => (),
            }
        }

        self.process_world_vertices(&mesh);

        // Reset the shader context's original active material.
        {
            let mut context = self.shader_context.write().unwrap();

            context.set_active_material(None);
        }
    }

    fn get_vertices_in(&self, mesh: &Mesh, face: &Face) -> [DefaultVertexIn; 3] {
        let v0_in = DefaultVertexIn {
            p: mesh.vertices[face.vertices.0].clone(),
            n: if face.normals.is_some() {
                mesh.normals[face.normals.unwrap().0].clone()
            } else {
                Default::default()
            },
            uv: if face.uvs.is_some() {
                mesh.uvs[face.uvs.unwrap().0].clone()
            } else {
                Default::default()
            },
            c: color::WHITE.to_vec3() / 255.0,
        };

        let v1_in = DefaultVertexIn {
            p: mesh.vertices[face.vertices.1].clone(),
            n: if face.normals.is_some() {
                mesh.normals[face.normals.unwrap().1].clone()
            } else {
                Default::default()
            },
            uv: if face.uvs.is_some() {
                mesh.uvs[face.uvs.unwrap().1].clone()
            } else {
                Default::default()
            },
            c: color::WHITE.to_vec3() / 255.0,
        };

        let v2_in = DefaultVertexIn {
            p: mesh.vertices[face.vertices.2].clone(),
            n: if face.normals.is_some() {
                mesh.normals[face.normals.unwrap().2].clone()
            } else {
                Default::default()
            },
            uv: if face.uvs.is_some() {
                mesh.uvs[face.uvs.unwrap().2].clone()
            } else {
                Default::default()
            },
            c: color::WHITE.to_vec3() / 255.0,
        };

        [v0_in, v1_in, v2_in]
    }

    fn process_world_vertices(&mut self, mesh: &Mesh) {
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

        let world_vertices = vertices_in
            .into_iter()
            .map(|v_in| return self.vertex_shader.call(&v_in))
            .collect();

        self.process_triangles(&mesh.faces, world_vertices);
    }

    fn process_triangles(&mut self, faces: &Vec<Face>, world_vertices: Vec<DefaultVertexOut>) {
        let mut triangles: Vec<Triangle<DefaultVertexOut>> = vec![];

        for face_index in 0..faces.len() {
            // Cull backfaces

            let v0 = world_vertices[face_index * 3];
            let v1 = world_vertices[face_index * 3 + 1];
            let v2 = world_vertices[face_index * 3 + 2];

            if self.options.cull_backfaces && self.is_backface(v0.p, v1.p, v2.p) {
                continue;
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
        if triangle.v0.p.x > triangle.v0.p.w
            && triangle.v1.p.x > triangle.v1.p.w
            && triangle.v2.p.x > triangle.v2.p.w
        {
            return true;
        }

        if triangle.v0.p.x < -triangle.v0.p.w
            && triangle.v1.p.x < -triangle.v1.p.w
            && triangle.v2.p.x < -triangle.v2.p.w
        {
            return true;
        }

        if triangle.v0.p.y > triangle.v0.p.w
            && triangle.v1.p.y > triangle.v1.p.w
            && triangle.v2.p.y > triangle.v2.p.w
        {
            return true;
        }

        if triangle.v0.p.y < -triangle.v0.p.w
            && triangle.v1.p.y < -triangle.v1.p.w
            && triangle.v2.p.y < -triangle.v2.p.w
        {
            return true;
        }

        if triangle.v0.p.z > triangle.v0.p.w
            && triangle.v1.p.z > triangle.v1.p.w
            && triangle.v2.p.z > triangle.v2.p.w
        {
            return true;
        }

        if triangle.v0.p.z < 0.0 && triangle.v1.p.z < 0.0 && triangle.v2.p.z < 0.0 {
            return true;
        }

        return false;
    }

    fn clip1(&mut self, v0: DefaultVertexOut, v1: DefaultVertexOut, v2: DefaultVertexOut) {
        let a_alpha = -(v0.p.z) / (v1.p.z - v0.p.z);
        let b_alpha = -(v0.p.z) / (v2.p.z - v0.p.z);

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
        let a_alpha = -(v0.p.z) / (v2.p.z - v0.p.z);
        let b_alpha = -(v1.p.z) / (v2.p.z - v1.p.z);

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

        if triangle.v0.p.z < 0.0 {
            if triangle.v1.p.z < 0.0 {
                // Clip 2 (0 and 1)
                self.clip2(triangle.v0, triangle.v1, triangle.v2);
            } else if triangle.v2.p.z < 0.0 {
                // Clip 2 (0 and 2)
                self.clip1(triangle.v0, triangle.v2, triangle.v1);
            } else {
                // Clip 1 (0)
                self.clip1(triangle.v0, triangle.v1, triangle.v2);
            }
        } else if triangle.v1.p.z < 0.0 {
            if triangle.v2.p.z < 0.0 {
                // Clip 2
                self.clip2(triangle.v1, triangle.v2, triangle.v0);
            } else {
                // Clip 1
                self.clip1(triangle.v1, triangle.v0, triangle.v2);
            }
        } else if triangle.v2.p.z < 0.0 {
            // Clip 1
            self.clip1(triangle.v2, triangle.v0, triangle.v1);
        } else {
            self.post_process_triangle_vertices(triangle);
        }
    }

    fn transform_to_ndc_space(&mut self, v: &mut DefaultVertexOut) {
        let w_inverse = 1.0 / v.p.w;

        *v *= w_inverse;

        v.p.x = (v.p.x + 1.0) * self.viewport.width_over_2;
        v.p.y = (-v.p.y + 1.0) * self.viewport.height_over_2;

        v.p.w = w_inverse;
    }

    fn post_process_triangle_vertices(&mut self, triangle: &mut Triangle<DefaultVertexOut>) {
        // World-space to screen-space (NDC) transform

        let world_vertices = [triangle.v0, triangle.v1, triangle.v2];

        let world_vertex_relative_normals = [
            world_vertices[0].p + world_vertices[0].n * 0.05,
            world_vertices[1].p + world_vertices[1].n * 0.05,
            world_vertices[2].p + world_vertices[2].n * 0.05,
        ];

        let mut screen_vertices = world_vertices.clone();

        self.transform_to_ndc_space(&mut screen_vertices[0]);
        self.transform_to_ndc_space(&mut screen_vertices[1]);
        self.transform_to_ndc_space(&mut screen_vertices[2]);

        // Interpolate entire vertex (all attributes) when drawing (scanline
        // interpolant)

        if self.options.show_rasterized_geometry {
            self.triangle_fill(screen_vertices[0], screen_vertices[1], screen_vertices[2]);
        }

        if self.options.show_wireframe {
            let mut points: Vec<Vec2> = vec![];

            for v in screen_vertices {
                points.push(Vec2 {
                    x: v.p.x,
                    y: v.p.y,
                    z: v.p.z,
                });
            }

            let wireframe_color = self.options.wireframe_color;

            Graphics::poly_line(
                &mut self.forward_framebuffer.as_mut().unwrap(),
                points.as_slice(),
                wireframe_color,
            );
        }

        if self.options.show_normals {
            for (index, v) in screen_vertices.iter().enumerate() {
                let world_vertex_relative_normal = world_vertex_relative_normals[index];

                let w_inverse = 1.0 / world_vertices[index].p.w;

                let screen_vertex_relative_normal = Vec2 {
                    x: (world_vertex_relative_normal.x * w_inverse + 1.0)
                        * self.viewport.width_over_2,
                    y: (-world_vertex_relative_normal.y * w_inverse + 1.0)
                        * self.viewport.height_over_2,
                    z: 0.0,
                };

                let from = v.p;
                let to = screen_vertex_relative_normal;

                Graphics::line(
                    &mut self.forward_framebuffer.as_mut().unwrap(),
                    from.x as i32,
                    from.y as i32,
                    to.x as i32,
                    to.y as i32,
                    color::RED,
                );
            }
        }
    }

    fn test_and_set_z_buffer(&mut self, x: u32, y: u32, interpolant: &mut DefaultVertexOut) {
        match self.z_buffer.as_mut().unwrap().test(x, y, interpolant.p.z) {
            Some(((x, y), non_linear_z)) => {
                let mut linear_space_interpolant = *interpolant * (1.0 / interpolant.p.w);

                if self.alpha_shader.call(&linear_space_interpolant) == false {
                    return;
                }

                self.z_buffer.as_mut().unwrap().set(x, y, non_linear_z);

                match self.g_buffer.as_mut() {
                    Some(g_buffer) => {
                        linear_space_interpolant.depth = non_linear_z;

                        g_buffer.set(x, y, self.geometry_shader.call(&linear_space_interpolant));
                    }
                    None => (),
                }
            }
            None => {}
        }
    }

    fn flat_top_triangle_fill(
        &mut self,
        top_left: DefaultVertexOut,
        top_right: DefaultVertexOut,
        bottom: DefaultVertexOut,
    ) {
        let delta_y = bottom.p.y - top_left.p.y;

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
        let delta_y = bottom_right.p.y - top.p.y;

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
        let y_start: u32 = u32::max((it0.p.y - 0.5).ceil() as u32, 0);
        let y_end: u32 = u32::min((it2.p.y - 0.5).ceil() as u32, self.viewport.height - 1);

        // Adjust both interpolants to account for us snapping y-start and y-end
        // to their nearest whole pixel coordinates.
        left_edge_interpolant += *left_step * (y_start as f32 + 0.5 - it0.p.y);
        *right_edge_interpolant += *right_step * (y_start as f32 + 0.5 - it0.p.y);

        // Rasterization loop
        for y in y_start..y_end {
            // Calculate our start and end X (end here is non-inclusive), such
            // that they are non-fractional screen coordinates.
            let x_start = u32::max((left_edge_interpolant.p.x - 0.5).ceil() as u32, 0);
            let x_end = u32::min(
                (right_edge_interpolant.p.x - 0.5).ceil() as u32,
                self.viewport.width - 1,
            );

            // Create an interpolant that we can move across our horizontal
            // scanline.
            let mut line_interpolant = left_edge_interpolant.clone();

            // Calculate the width of our scanline, for this Y position.
            let dx = right_edge_interpolant.p.x - left_edge_interpolant.p.x;

            // Calculate the change (step) for our horizontal interpolant, based
            // on the width of our scanline.
            let line_interpolant_step = (*right_edge_interpolant - line_interpolant) / dx;

            // Prestep our scanline interpolant to account for us snapping
            // x-start and x-end to their nearest whole pixel coordinates.
            line_interpolant +=
                line_interpolant_step * ((x_start as f32) + 0.5 - left_edge_interpolant.p.x);

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

        if tri[1].p.y < tri[0].p.y {
            tri.swap(0, 1);
        }
        if tri[2].p.y < tri[1].p.y {
            tri.swap(1, 2);
        }
        if tri[1].p.y < tri[0].p.y {
            tri.swap(0, 1);
        }

        if tri[0].p.y == tri[1].p.y {
            // Flat-top (horizontal line is tri[0]-to-tri[1]);

            // tri[2] must sit below tri[0] and tri[1]; tri[0] and tri[1] cannot
            // have the same x-value; therefore, sort tri[0] and tri[1] by x-value;

            if tri[1].p.x < tri[0].p.x {
                tri.swap(0, 1);
            }

            self.flat_top_triangle_fill(tri[0], tri[1], tri[2]);
        } else if tri[1].p.y == tri[2].p.y {
            // Flat-bottom (horizontal line is tri[1]-to-tri[2]);

            // tri[0] must sit above tri[1] and tri[2]; tri[1] and tri[2] cannot
            // have the same x-value; therefore, sort tri[1] and tri[2] by x-value;

            if tri[2].p.x < tri[1].p.x {
                tri.swap(1, 2);
            }

            self.flat_bottom_triangle_fill(tri[0], tri[1], tri[2]);
        } else {
            // Find splitting vertex

            let alpha_split = (tri[1].p.y - tri[0].p.y) / (tri[2].p.y - tri[0].p.y);

            let split_vertex = DefaultVertexOut::interpolate(tri[0], tri[2], alpha_split);

            if tri[1].p.x < split_vertex.p.x {
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

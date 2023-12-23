use crate::{
    material::Material,
    mesh::Face,
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

use super::{
    color::{self, Color},
    effect::Effect,
    graphics::Graphics,
    mesh::Mesh,
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
};

#[derive(Copy, Clone, Default)]
struct Triangle<T> {
    v0: T,
    v1: T,
    v2: T,
}

#[derive(Copy, Clone, Default)]
pub struct PipelineOptions {
    pub should_render_wireframe: bool,
    pub should_render_shader: bool,
    pub should_render_normals: bool,
    pub should_cull_backfaces: bool,
}

pub struct Pipeline<T> {
    options: PipelineOptions,
    graphics: Graphics,
    buffer_width_over_2: f32,
    buffer_height_over_2: f32,
    z_buffer: Vec<f32>,
    pub effect: T,
}

impl<'a, T: Effect<VertexIn = DefaultVertexIn, VertexOut = DefaultVertexOut>> Pipeline<T>
where
    T: Effect,
{
    pub fn new(graphics: Graphics, effect: T, options: PipelineOptions) -> Self {
        let z_buffer_size: usize = (graphics.buffer.width * graphics.buffer.height) as usize;

        let mut z_buffer: Vec<f32> = Vec::with_capacity(z_buffer_size);

        for _ in 0..z_buffer_size {
            z_buffer.push(f32::MAX);
        }

        let buffer_width_over_2 = (graphics.buffer.width as f32) / 2.0;
        let buffer_height_over_2 = (graphics.buffer.height as f32) / 2.0;

        return Pipeline {
            options,
            graphics,
            buffer_width_over_2: buffer_width_over_2,
            buffer_height_over_2: buffer_height_over_2,
            z_buffer,
            effect,
        };
    }

    pub fn get_pixel_data(&self) -> &Vec<u32> {
        return self.graphics.get_pixel_data();
    }

    pub fn set_options(&mut self, options: PipelineOptions) {
        self.options = options;
    }

    pub fn render_mesh(&mut self, mesh: &Mesh) {
        match &mesh.material {
            Some(mat) => {
                // Set the pipeline effect's active material to this mesh's
                // material
                let mat_raw_mut = &*mat as *const Material;

                self.effect.set_active_material(Some(mat_raw_mut));
            }
            None => (),
        }

        self.process_world_vertices(&mesh);

        // Reset the pipeline effect's active material
        self.effect.set_active_material(None);
    }

    pub fn clear_pixel_buffer(&mut self) {
        self.graphics.buffer.clear(color::BLACK);
    }

    pub fn clear_z_buffer(&mut self) {
        for i in 0..self.z_buffer.len() {
            self.z_buffer[i] = f32::MAX;
        }
    }

    fn process_world_vertices(&mut self, mesh: &Mesh) {
        // Map each face to a set of 3 unique instances of DefaultVertexIn.

        let mut vertices_in: Vec<DefaultVertexIn> = vec![];

        for face_index in 0..mesh.faces.len() {
            let face = mesh.faces[face_index];

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

            vertices_in.push(v0_in);
            vertices_in.push(v1_in);
            vertices_in.push(v2_in);
        }

        // Process mesh vertices from object-space to world-space.

        let world_vertices = vertices_in
            .into_iter()
            .map(|v_in| return self.effect.vs(v_in))
            .collect();

        self.process_triangles(&mesh.faces, world_vertices);
    }

    fn process_triangles(&mut self, faces: &Vec<Face>, world_vertices: Vec<T::VertexOut>) {
        let mut triangles: Vec<Triangle<T::VertexOut>> = vec![];

        for face_index in 0..faces.len() {
            // Cull backfaces

            let v0 = world_vertices[face_index * 3];
            let v1 = world_vertices[face_index * 3 + 1];
            let v2 = world_vertices[face_index * 3 + 2];

            if self.options.should_cull_backfaces && self.is_backface(v0.p, v1.p, v2.p) {
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

        let projected_origin = Vec4::new(Default::default(), 1.0) * self.effect.get_projection();

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
        triangle: &mut Triangle<T::VertexOut>,
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

    fn clip1(&mut self, v0: T::VertexOut, v1: T::VertexOut, v2: T::VertexOut) {
        let a_alpha = -(v0.p.z) / (v1.p.z - v0.p.z);
        let b_alpha = -(v0.p.z) / (v2.p.z - v0.p.z);

        let a_prime = T::VertexOut::interpolate(v0, v1, a_alpha);
        let b_prime = T::VertexOut::interpolate(v0, v2, b_alpha);

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

    fn clip2(&mut self, v0: T::VertexOut, v1: T::VertexOut, v2: T::VertexOut) {
        let a_alpha = -(v0.p.z) / (v2.p.z - v0.p.z);
        let b_alpha = -(v1.p.z) / (v2.p.z - v1.p.z);

        let a_prime = T::VertexOut::interpolate(v0, v2, a_alpha);
        let b_prime = T::VertexOut::interpolate(v1, v2, b_alpha);

        let mut triangle = Triangle {
            v0: a_prime,
            v1: b_prime,
            v2,
        };

        self.post_process_triangle_vertices(&mut triangle);
    }

    fn process_triangle(&mut self, triangle: &mut Triangle<T::VertexOut>) {
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

    fn transform_to_ndc_space(&mut self, v: &mut T::VertexOut) {
        let w_inverse = 1.0 / v.p.w;

        *v *= w_inverse;

        v.p.x = (v.p.x + 1.0) * self.buffer_width_over_2;
        v.p.y = (-v.p.y + 1.0) * self.buffer_height_over_2;

        v.p.w = w_inverse;
    }

    fn post_process_triangle_vertices(&mut self, triangle: &mut Triangle<T::VertexOut>) {
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

        if self.options.should_render_shader {
            self.triangle_fill(screen_vertices[0], screen_vertices[1], screen_vertices[2]);
        }

        if self.options.should_render_wireframe {
            let mut points: Vec<Vec2> = vec![];

            for v in screen_vertices {
                points.push(Vec2 {
                    x: v.p.x,
                    y: v.p.y,
                    z: v.p.z,
                });
            }

            let mut c = color::WHITE;

            if self.options.should_cull_backfaces == false {
                c = Color {
                    r: (world_vertices[0].c.x) as u8,
                    g: (world_vertices[0].c.y) as u8,
                    b: (world_vertices[0].c.z) as u8,
                    a: 255,
                };
            }

            self.graphics.poly_line(points.as_slice(), c);
        }

        if self.options.should_render_normals {
            for (index, v) in screen_vertices.iter().enumerate() {
                let world_vertex_relative_normal = world_vertex_relative_normals[index];

                let w_inverse = 1.0 / world_vertices[index].p.w;

                let screen_vertex_relative_normal = Vec2 {
                    x: (world_vertex_relative_normal.x * w_inverse + 1.0)
                        * self.buffer_width_over_2,
                    y: (-world_vertex_relative_normal.y * w_inverse + 1.0)
                        * self.buffer_height_over_2,
                    z: 0.0,
                };

                let from = v.p;
                let to = screen_vertex_relative_normal;

                self.graphics.line(
                    from.x as u32,
                    from.y as u32,
                    to.x as u32,
                    to.y as u32,
                    color::RED,
                );
            }
        }
    }

    fn test_and_set_z_buffer(&mut self, x: u32, y: u32, z: f32) -> bool {
        let z_buffer_index = (y * self.graphics.buffer.width + x) as usize;

        if z_buffer_index >= self.z_buffer.len() {
            panic!(
                "Call to draw::set_pixel with invalid coordinate ({},{})!",
                x, y
            );
        }

        if z < self.z_buffer[z_buffer_index] {
            self.z_buffer[z_buffer_index] = z;
            return true;
        } else {
            return false;
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, z: f32, interpolant: &mut T::VertexOut) {
        if x > (self.graphics.buffer.width - 1)
            || y > (self.graphics.buffer.pixels.len() as u32 / self.graphics.buffer.width as u32
                - 1)
        {
            // Prevents panic! inside of self.graphics.set_pixel();

            return;
        }

        if self.test_and_set_z_buffer(x, y, interpolant.p.z) {
            let linear_space_interpolant = *interpolant * (1.0 / interpolant.p.w);

            let color = self.effect.ps(&linear_space_interpolant);

            self.graphics.set_pixel(x, y, color);
        }
    }

    fn flat_top_triangle_fill(&mut self, v0: T::VertexOut, v1: T::VertexOut, v2: T::VertexOut) {
        // @NOTE(mzalla) v0 as top-left vertex
        // @NOTE(mzalla) v1 as top-right vertex
        // @NOTE(mzalla) v2 as bottom vertex

        let left_step_x = (v2.p.x - v0.p.x) / (v2.p.y - v0.p.y);
        let right_step_x = (v2.p.x - v1.p.x) / (v2.p.y - v1.p.y);

        let left_step_z = (v2.p.z - v0.p.z) / (v2.p.y - v0.p.y);
        let right_step_z = (v2.p.z - v1.p.z) / (v2.p.y - v1.p.y);

        let y_start = ((v0.p.y - 0.5).ceil() as u32)
            .max(0)
            .min(self.graphics.buffer.height - 1);
        let y_end = ((v2.p.y - 0.5).ceil() as u32).min(self.graphics.buffer.height - 1);

        let mut lhs = v0.clone();
        let lhs_step = (v2 - v0) / (y_end - y_start) as f32;

        let mut rhs = v1.clone();
        let rhs_step = (v2 - v1) / (y_end - y_start) as f32;

        for y in y_start..y_end {
            let delta_y = y as f32 + 0.5 - v0.p.y;

            let x_left = v0.p.x + left_step_x * delta_y;
            let x_right = v1.p.x + right_step_x * delta_y;
            let x_span = x_right - x_left;

            let z_start: f32 = v0.p.z + left_step_z * delta_y;
            let z_end: f32 = v1.p.z + right_step_z * delta_y;
            let z_span: f32 = z_end - z_start;

            let x_start = ((x_left - 0.5).ceil() as u32)
                .max(0)
                .min(self.graphics.buffer.width - 1);
            let x_end = ((x_right - 0.5).ceil() as u32).min(self.graphics.buffer.width - 1);

            let mut cursor = lhs.clone();
            let cursor_step = (rhs - cursor) / (x_end - x_start) as f32;

            for x in x_start..x_end {
                let x_relative = x - x_start;
                let x_progress: f32 = x_relative as f32 / x_span as f32;

                let z = z_start + z_span * x_progress;

                self.set_pixel(x, y, z, &mut cursor);

                cursor = cursor + cursor_step;
            }

            lhs = lhs + lhs_step;
            rhs = rhs + rhs_step;
        }
    }

    fn flat_bottom_triangle_fill(&mut self, v0: T::VertexOut, v1: T::VertexOut, v2: T::VertexOut) {
        // @NOTE(mzalla) v0 as top vertex
        // @NOTE(mzalla) v1 as bottom-left vertex
        // @NOTE(mzalla) v2 as bottom-right vertex

        let left_step_x = (v1.p.x - v0.p.x) / (v1.p.y - v0.p.y);
        let right_step_x = (v2.p.x - v0.p.x) / (v2.p.y - v0.p.y);

        let left_step_z = (v1.p.z - v0.p.z) / (v1.p.y - v0.p.y);
        let right_step_z = (v2.p.z - v0.p.z) / (v2.p.y - v0.p.y);

        let y_start = ((v0.p.y - 0.5).ceil() as u32)
            .max(0)
            .min(self.graphics.buffer.height - 1);
        let y_end = ((v2.p.y - 0.5).ceil() as u32).min(self.graphics.buffer.height - 1);

        let mut lhs = v0.clone();
        let lhs_step = (v1 - v0) / (y_end - y_start) as f32;

        let mut rhs = v0.clone();
        let rhs_step = (v2 - v0) / (y_end - y_start) as f32;

        for y in y_start..y_end {
            let delta_y = y as f32 + 0.5 - v0.p.y;

            let x_left = v0.p.x + left_step_x * delta_y;
            let x_right = v0.p.x + right_step_x * delta_y;
            let x_span = x_right - x_left;

            let z_start: f32 = v0.p.z + left_step_z * delta_y;
            let z_end: f32 = v0.p.z + right_step_z * delta_y;
            let z_span: f32 = z_end - z_start;

            let x_start = ((x_left - 0.5).ceil() as u32)
                .max(0)
                .min(self.graphics.buffer.width - 1);
            let x_end = ((x_right - 0.5).ceil() as u32).min(self.graphics.buffer.width - 1);

            let mut cursor = lhs.clone();
            let cursor_step = (rhs - cursor) / (x_end - x_start) as f32;

            for x in x_start..x_end {
                let x_relative = x - x_start;
                let x_progress: f32 = x_relative as f32 / x_span as f32;

                let z = z_start + z_span * x_progress;

                self.set_pixel(x, y, z, &mut cursor);

                cursor = cursor + cursor_step;
            }

            lhs = lhs + lhs_step;
            rhs = rhs + rhs_step;
        }
    }

    fn triangle_fill(&mut self, v0: T::VertexOut, v1: T::VertexOut, v2: T::VertexOut) {
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

            let split_vertex = T::VertexOut::interpolate(tri[0], tri[2], alpha_split);

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

use std::sync::RwLock;

use crate::{
    entity::Entity,
    material::{cache::MaterialCache, Material},
    matrix::Mat4,
    mesh::{self, primitive::cube, Face},
    scene::{
        camera::Camera,
        light::{PointLight, SpotLight},
    },
    shader::{alpha::AlphaShader, fragment::FragmentShader, vertex::VertexShader, ShaderContext},
    shaders::{
        default_alpha_shader::DefaultAlphaShader, default_fragment_shader::DefaultFragmentShader,
        default_vertex_shader::DefaultVertexShader,
    },
    texture::cubemap::CubeMap,
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

use self::{options::PipelineOptions, zbuffer::ZBuffer};

use super::{
    color::{self, Color},
    graphics::Graphics,
    mesh::Mesh,
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
};

pub mod options;
mod zbuffer;

#[derive(Copy, Clone, Default)]
struct Triangle<T> {
    v0: T,
    v1: T,
    v2: T,
}

pub struct Pipeline<
    'a,
    V = DefaultVertexShader<'a>,
    A = DefaultAlphaShader<'a>,
    F = DefaultFragmentShader<'a>,
> where
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    F: FragmentShader<'a>,
{
    pub options: PipelineOptions,
    graphics: Graphics,
    buffer_width_over_2: f32,
    buffer_height_over_2: f32,
    z_buffer: ZBuffer,
    pub shader_context: &'a RwLock<ShaderContext>,
    vertex_shader: V,
    alpha_shader: A,
    pub fragment_shader: F,
}

impl<'a, V, A, F> Pipeline<'a, V, A, F>
where
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    F: FragmentShader<'a>,
{
    pub fn new(
        graphics: Graphics,
        projection_z_near: f32,
        projection_z_far: f32,
        shader_context: &'a RwLock<ShaderContext>,
        vertex_shader: V,
        fragment_shader: F,
        options: PipelineOptions,
    ) -> Self {
        let z_buffer = ZBuffer::new(
            graphics.buffer.width,
            graphics.buffer.height,
            projection_z_near,
            projection_z_far,
        );

        let buffer_width_over_2 = (graphics.buffer.width as f32) / 2.0;
        let buffer_height_over_2 = (graphics.buffer.height as f32) / 2.0;

        let alpha_shader = AlphaShader::new(shader_context);

        return Pipeline {
            options,
            graphics,
            buffer_width_over_2: buffer_width_over_2,
            buffer_height_over_2: buffer_height_over_2,
            z_buffer,
            shader_context,
            vertex_shader,
            alpha_shader,
            fragment_shader,
        };
    }

    pub fn get_pixel_data(&'a self) -> &'a Vec<u32> {
        return self.graphics.buffer.get_pixel_data();
    }

    pub fn begin_frame(&mut self) {
        self.graphics.buffer.clear(color::BLACK);

        self.z_buffer.clear();
    }

    pub fn render_point(
        &mut self,
        position: Vec3,
        color: Color,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
        material_name: Option<String>,
        scale: Option<f32>,
    ) {
        let vertex_in = DefaultVertexIn {
            p: position,
            c: color.to_vec3() / 255.0,
            ..Default::default()
        };

        let mut vertex_out = self.vertex_shader.call(&vertex_in);

        self.transform_to_ndc_space(&mut vertex_out);

        let x = vertex_out.p.x as u32;
        let y = vertex_out.p.y as u32;

        match material_cache {
            Some(materials) => {
                let mat_name = material_name.unwrap();
                let billboard_scale = scale.unwrap();

                let mut quad = mesh::primitive::billboard::generate(
                    camera.unwrap(),
                    billboard_scale,
                    billboard_scale,
                );

                let light_mat = materials.get_mut(&mat_name);

                match light_mat {
                    Some(material) => {
                        material.diffuse_color = color.to_vec3() / 255.0;

                        quad.material_name = Some(mat_name.clone());

                        let mut light_quad_entity = Entity::new(&quad);

                        light_quad_entity.position = position;

                        self.render_entity(&light_quad_entity, Some(materials));
                    }
                    None => {
                        // @TODO Clip to view frustum
                        self.graphics.buffer.set_pixel(x, y, color);
                    }
                }
            }
            None => {
                // @TODO Clip to view frustum
                self.graphics.buffer.set_pixel(x, y, color);
            }
        }
    }

    pub fn render_line(&mut self, start: Vec3, end: Vec3, color: Color) {
        // @TODO Clip to view frustum

        let start_vertex_in = DefaultVertexIn {
            p: start,
            c: color.to_vec3() / 255.0,
            ..Default::default()
        };

        let end_vertex_in = DefaultVertexIn {
            p: end,
            c: color.to_vec3() / 255.0,
            ..Default::default()
        };

        let mut start_vertex_out = self.vertex_shader.call(&start_vertex_in);
        let mut end_vertex_out = self.vertex_shader.call(&end_vertex_in);

        self.render_line_2(&mut start_vertex_out, &mut end_vertex_out, color);
    }

    pub fn render_line_2(
        &mut self,
        start: &mut DefaultVertexOut,
        end: &mut DefaultVertexOut,
        color: Color,
    ) {
        // @TODO Clip to view frustum

        self.transform_to_ndc_space(start);
        self.transform_to_ndc_space(end);

        self.graphics.line(
            start.p.x as i32,
            start.p.y as i32,
            end.p.x as i32,
            end.p.y as i32,
            color,
        );
    }

    fn render_point_indicator(&mut self, position: Vec3, scale: f32) {
        // X-axis (red)

        self.render_line(
            Vec3 {
                x: -1.0 * scale,
                y: 0.0,
                z: 0.0,
            } + position,
            Vec3 {
                x: 1.0 * scale,
                y: 0.0,
                z: 0.0,
            } + position,
            color::RED,
        );

        // Y-axis (blue)

        self.render_line(
            Vec3 {
                x: 0.0,
                y: -1.0 * scale,
                z: 0.0,
            } + position,
            Vec3 {
                x: 0.0,
                y: 1.0 * scale,
                z: 0.0,
            } + position,
            color::BLUE,
        );

        // Z-axis (green)

        self.render_line(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0 * scale,
            } + position,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0 * scale,
            } + position,
            color::GREEN,
        );
    }

    pub fn render_world_axes(&mut self, scale: f32) {
        self.render_point_indicator(Default::default(), scale)
    }

    pub fn render_camera(&mut self, camera: &Camera) {
        let origin = camera.get_position();

        // Target

        self.render_line(origin, camera.get_target(), color::WHITE);

        let aspect_ratio = camera.get_aspect_ratio();

        let right_for_aspect_ratio = camera.get_right() * aspect_ratio;

        // Top
        self.render_line(
            origin + camera.get_up() - right_for_aspect_ratio,
            origin + camera.get_up() + right_for_aspect_ratio,
            color::WHITE,
        );

        // Bottom
        self.render_line(
            origin - camera.get_up() - right_for_aspect_ratio,
            origin - camera.get_up() + right_for_aspect_ratio,
            color::WHITE,
        );

        // Left
        self.render_line(
            origin - right_for_aspect_ratio - camera.get_up(),
            origin - right_for_aspect_ratio + camera.get_up(),
            color::WHITE,
        );

        // Right
        self.render_line(
            origin + right_for_aspect_ratio - camera.get_up(),
            origin + right_for_aspect_ratio + camera.get_up(),
            color::WHITE,
        );
    }

    fn render_light(
        &mut self,
        light_position: Vec3,
        light_intensities: Vec3,
        light_influence_distance: f32,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
        is_spot_light: bool,
    ) {
        match material_cache {
            Some(materials) => {
                let light_material_name = if is_spot_light {
                    "spot_light_decal"
                } else {
                    "point_light_decal"
                };

                let billboard_scale: f32 = if is_spot_light { 1.25 } else { 0.75 };

                let mut light_quad = mesh::primitive::billboard::generate(
                    camera.unwrap(),
                    billboard_scale,
                    billboard_scale,
                );

                let light_mat = materials.get_mut(&light_material_name.to_string());

                match light_mat {
                    Some(material) => {
                        material.diffuse_color = light_intensities;

                        light_quad.material_name = Some(light_material_name.to_string());

                        let mut light_quad_entity = Entity::new(&light_quad);

                        light_quad_entity.position = light_position;

                        let world_transform = Mat4::scaling(1.0)
                            * Mat4::rotation_x(light_quad_entity.rotation.x)
                            * Mat4::rotation_y(light_quad_entity.rotation.y)
                            * Mat4::rotation_z(light_quad_entity.rotation.z)
                            * Mat4::translation(light_quad_entity.position);

                        {
                            let mut context = self.shader_context.write().unwrap();

                            context.set_world_transform(world_transform);
                        }

                        self.render_mesh(&light_quad_entity.mesh, Some(materials));
                    }
                    None => {
                        self.render_point_indicator(light_position, light_influence_distance * 0.2);
                    }
                }
            }
            None => {
                self.render_point_indicator(light_position, light_influence_distance * 0.2);
            }
        }
    }

    pub fn render_point_light(
        &mut self,
        light: &PointLight,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
    ) {
        self.render_light(
            light.position,
            light.intensities,
            light.influence_distance,
            camera,
            material_cache,
            false,
        );
    }

    pub fn render_spot_light(
        &mut self,
        light: &SpotLight,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
    ) {
        self.render_light(
            light.position,
            light.intensities,
            light.influence_distance,
            camera,
            material_cache,
            true,
        );

        let start = light.position;
        let end = light.position + light.direction.as_normal() * light.influence_distance;

        self.render_line(start, end, color::WHITE);

        // Draw sides for cutoff angles.

        let down_normal = Vec4::new((end - start).as_normal(), 1.0);

        let mut draw_sides = |cutoff_angle: f32, cutoff_angle_cos: f32, color: Color| {
            let hypotenuse_ratio = 1.0 / cutoff_angle_cos;

            let normal_rotated_x = (down_normal * Mat4::rotation_x(cutoff_angle)).as_normal();
            let normal_rotated_neg_x = (down_normal * Mat4::rotation_x(-cutoff_angle)).as_normal();

            let x = normal_rotated_x.to_vec3() * hypotenuse_ratio * light.influence_distance;
            let neg_x =
                normal_rotated_neg_x.to_vec3() * hypotenuse_ratio * light.influence_distance;

            let normal_rotated_z = (down_normal * Mat4::rotation_z(cutoff_angle)).as_normal();
            let normal_rotated_neg_z = (down_normal * Mat4::rotation_z(-cutoff_angle)).as_normal();

            let z = normal_rotated_z.to_vec3() * hypotenuse_ratio * light.influence_distance;
            let neg_z =
                normal_rotated_neg_z.to_vec3() * hypotenuse_ratio * light.influence_distance;

            self.render_line(start, start + x, color);
            self.render_line(start, start + neg_x, color);

            self.render_line(start, start + z, color);
            self.render_line(start, start + neg_z, color);

            self.render_line(start + x, start + z, color);
            self.render_line(start + z, start + neg_x, color);
            self.render_line(start + neg_x, start + neg_z, color);
            self.render_line(start + neg_z, start + x, color);
        };

        draw_sides(
            light.outer_cutoff_angle,
            light.outer_cutoff_angle_cos,
            color::YELLOW,
        );
    }

    pub fn render_entity(&mut self, entity: &Entity, material_cache: Option<&MaterialCache>) {
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

        self.render_mesh(&entity.mesh, material_cache);

        // Reset the shader context's original world transform.
        {
            let mut context = self.shader_context.write().unwrap();

            context.set_world_transform(original_world_transform);
        }
    }

    pub fn render_mesh(&mut self, mesh: &Mesh, material_cache: Option<&MaterialCache>) {
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

    pub fn render_skybox(&mut self, skybox: &CubeMap, camera: &Camera) {
        for (index, z_non_linear) in self.z_buffer.values.iter().enumerate() {
            // If this pixel was not shaded by our fragment shader

            if *z_non_linear == zbuffer::MAX_DEPTH {
                // Note: z_buffer_index = (y * self.graphics.buffer.width + x)

                let screen_x: u32 = (index as f32 % self.graphics.buffer.width as f32) as u32;
                let screen_y: u32 = (index as f32 / self.graphics.buffer.width as f32) as u32;

                let pixel_coordinate_world_space = camera.get_pixel_world_space_position(
                    screen_x,
                    screen_y,
                    self.graphics.buffer.width,
                    self.graphics.buffer.height,
                );

                let normal = pixel_coordinate_world_space.as_normal();

                // Sample the cubemap using our world-space direction-offset.

                let skybox_color = skybox.sample(&normal);

                self.graphics
                    .buffer
                    .set_pixel(screen_x, screen_y, skybox_color);
            }
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

        v.p.x = (v.p.x + 1.0) * self.buffer_width_over_2;
        v.p.y = (-v.p.y + 1.0) * self.buffer_height_over_2;

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
                    from.x as i32,
                    from.y as i32,
                    to.x as i32,
                    to.y as i32,
                    color::RED,
                );
            }
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, interpolant: &mut DefaultVertexOut) {
        if x > (self.graphics.buffer.width - 1) || y > (self.graphics.buffer.height as u32 - 1) {
            // Prevents panic! inside of self.graphics.buffer.set_pixel();
            return;
        }

        match self.z_buffer.test(x, y, interpolant.p.z) {
            Some((index, non_linear_z)) => {
                let mut linear_space_interpolant = *interpolant * (1.0 / interpolant.p.w);

                if self.alpha_shader.call(&linear_space_interpolant) == false {
                    return;
                }

                self.z_buffer.set(index, non_linear_z);

                linear_space_interpolant.depth = non_linear_z;

                self.graphics.buffer.set_pixel(
                    x,
                    y,
                    self.fragment_shader.call(&linear_space_interpolant),
                );
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
        let y_end: u32 = u32::min(
            (it2.p.y - 0.5).ceil() as u32,
            self.graphics.buffer.height - 1,
        );

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
                self.graphics.buffer.width - 1,
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
                self.set_pixel(x, y, &mut line_interpolant);

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

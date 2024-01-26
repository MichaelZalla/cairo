use crate::{
    color::Color,
    entity::Entity,
    material::cache::MaterialCache,
    mesh,
    pipeline::Pipeline,
    scene::camera::Camera,
    shader::{
        alpha::AlphaShader, fragment::FragmentShader, geometry::GeometryShader,
        vertex::VertexShader,
    },
    vec::vec3::Vec3,
    vertex::default_vertex_in::DefaultVertexIn,
};

impl<'a, F, V, A, G> Pipeline<'a, F, V, A, G>
where
    F: FragmentShader<'a>,
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
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
        let z = vertex_out.p.z;

        // Cull points that are in front of our near plane (z <= 0).
        if z <= 0.0 {
            return;
        }

        let color_u32 = color.to_u32();

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
                        self.forward_framebuffer
                            .as_mut()
                            .unwrap()
                            .set(x, y, color_u32);
                    }
                }
            }
            None => {
                self.forward_framebuffer
                    .as_mut()
                    .unwrap()
                    .set(x, y, color_u32);
            }
        }
    }
}

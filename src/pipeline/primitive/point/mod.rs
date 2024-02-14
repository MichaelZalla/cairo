use crate::{
    color::Color, entity::Entity, material::cache::MaterialCache, mesh, pipeline::Pipeline,
    scene::camera::Camera, vec::vec3::Vec3,
};

impl<'a> Pipeline<'a> {
    pub fn render_point(
        &mut self,
        point_world_space: Vec3,
        color: Color,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
        material_name: Option<String>,
        scale: Option<f32>,
    ) {
        let shader_context = self.shader_context.read().unwrap();

        let point_ndc_space = shader_context.to_ndc_space(point_world_space);

        let x = (point_ndc_space.x * self.viewport.width as f32) as u32;
        let y = (point_ndc_space.y * self.viewport.height as f32) as u32;
        let z = point_ndc_space.z;

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
                    point_world_space,
                    &camera.unwrap().look_vector.get_position(),
                    billboard_scale,
                    billboard_scale,
                );

                let light_mat = materials.get_mut(&mat_name);

                match light_mat {
                    Some(material) => {
                        material.diffuse_color = color.to_vec3() / 255.0;

                        quad.material_name = Some(mat_name.clone());

                        let light_quad_entity = Entity::new(&quad);

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

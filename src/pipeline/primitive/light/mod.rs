use crate::{
    color::{self},
    entity::Entity,
    material::cache::MaterialCache,
    mesh,
    pipeline::Pipeline,
    scene::{
        camera::Camera,
        light::{PointLight, SpotLight},
    },
    vec::vec3::Vec3,
};

impl<'a> Pipeline<'a> {
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
                    light_position,
                    &camera.unwrap().look_vector.get_position(),
                    billboard_scale,
                    billboard_scale,
                );

                let light_mat = materials.get_mut(&light_material_name.to_string());

                match light_mat {
                    Some(material) => {
                        material.diffuse_color = light_intensities;

                        light_quad.material_name = Some(light_material_name.to_string());

                        let light_quad_entity = Entity::new(&light_quad);

                        self.render_entity(&light_quad_entity, Some(&materials));
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
        let light_position = light.look_vector.get_position();

        self.render_light(
            light_position,
            light.intensities,
            light.influence_distance,
            camera,
            material_cache,
            true,
        );

        let target_position =
            light_position + light.look_vector.get_forward().as_normal() * light.influence_distance;

        self.render_line(light_position, target_position, color::WHITE);

        // Draw sides for cutoff angles.

        let opposite_over_adjacent = light.outer_cutoff_angle.tan();

        let near_plane_points_world_space = [
            light_position,
            light_position,
            light_position,
            light_position,
        ];

        let far_plane_points_world_space = [
            target_position
                + light.look_vector.get_right() * opposite_over_adjacent * light.influence_distance,
            target_position
                + light.look_vector.get_up()
                    * -1.0
                    * opposite_over_adjacent
                    * light.influence_distance,
            target_position
                + light.look_vector.get_right()
                    * -1.0
                    * opposite_over_adjacent
                    * light.influence_distance,
            target_position
                + light.look_vector.get_up() * opposite_over_adjacent * light.influence_distance,
        ];

        self.render_frustum(near_plane_points_world_space, far_plane_points_world_space);
    }
}

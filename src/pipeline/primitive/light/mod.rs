use crate::{
    color::{self, Color},
    entity::Entity,
    material::cache::MaterialCache,
    matrix::Mat4,
    mesh,
    pipeline::Pipeline,
    scene::{
        camera::Camera,
        light::{PointLight, SpotLight},
    },
    shader::geometry::GeometryShader,
    vec::{vec3::Vec3, vec4::Vec4},
};

impl<'a, G> Pipeline<'a, G>
where
    G: GeometryShader<'a>,
{
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
}

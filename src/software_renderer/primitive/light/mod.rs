use crate::{
    color::{self, Color},
    render::Renderer,
    scene::{
        camera::frustum::Frustum,
        light::{point_light::PointLight, spot_light::SpotLight},
    },
    software_renderer::SoftwareRenderer,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_point_light(&mut self, light: &PointLight) {
        self.render_point_indicator(light.position, light.influence_distance);
    }

    pub(in crate::software_renderer) fn _render_spot_light(&mut self, light: &SpotLight) {
        let light_position = light.look_vector.get_position();

        // self.render_point_indicator(light.look_vector.get_position(), light.influence_distance);

        let forward = light.look_vector.get_forward().as_normal();

        let target_position = light_position + forward * light.influence_distance;

        self.render_line(light_position, target_position, &color::WHITE);

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

        // Exposure tone mapping

        let mut color = light.intensities.tone_map_exposure(1.0);

        color.linear_to_srgb();

        let frustum = Frustum {
            forward,
            near: near_plane_points_world_space,
            far: far_plane_points_world_space,
        };

        self._render_frustum(&frustum, Some(&Color::from_vec3(color * 255.0)));
    }
}

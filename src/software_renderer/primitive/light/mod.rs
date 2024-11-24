use crate::{
    color,
    matrix::Mat4,
    render::Renderer,
    scene::{
        camera::frustum::Frustum,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
    },
    software_renderer::SoftwareRenderer,
    vec::{vec3::Vec3, vec4::Vec4},
};

impl SoftwareRenderer {
    fn render_light_ground_contact(&mut self, position: &Vec3) {
        let (y_indicator_line_start, y_indicator_line_end) = (
            Vec3 {
                y: position.y,
                ..*position
            },
            Vec3 {
                y: 0.0,
                ..*position
            },
        );

        self.render_line(
            y_indicator_line_start,
            y_indicator_line_end,
            color::LIGHT_GRAY,
        );
    }

    pub(in crate::software_renderer) fn _render_ambient_light(
        &mut self,
        transform: &Mat4,
        light: &AmbientLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        let color = self.get_tone_mapped_color_from_hdr(light.intensities);

        self.render_circle(&position, 1.0, color);
    }

    pub(in crate::software_renderer) fn _render_directional_light(
        &mut self,
        transform: &Mat4,
        light: &DirectionalLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        let color = self.get_tone_mapped_color_from_hdr(light.intensities);

        self.render_circle(&position, 1.0, color);

        let color = self.get_tone_mapped_color_from_hdr(light.intensities);

        let (start, end) = (position, position + light.get_direction().to_vec3() * 10.0);

        self.render_line(start, end, color);
    }

    pub(in crate::software_renderer) fn _render_point_light(
        &mut self,
        transform: &Mat4,
        light: &PointLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        let radius = light.influence_distance;

        let color = self.get_tone_mapped_color_from_hdr(light.intensities);

        self.render_circle(&position, 1.0, color);
        self.render_circle(&position, radius, color);
    }

    pub(in crate::software_renderer) fn _render_spot_light(
        &mut self,
        transform: &Mat4,
        light: &SpotLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        let color = self.get_tone_mapped_color_from_hdr(light.intensities);

        self.render_circle(&position, 1.0, color);

        let forward = light.look_vector.get_forward().as_normal();

        let target_position = position + forward * light.influence_distance;

        self.render_line(position, target_position, color::WHITE);

        // Draw sides for cutoff angles.

        let opposite_over_adjacent = light.outer_cutoff_angle.tan();

        let near_plane_points_world_space = [position, position, position, position];

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

        let color = self.get_tone_mapped_color_from_hdr(light.intensities);

        let (near, far) = (near_plane_points_world_space, far_plane_points_world_space);

        let frustum = Frustum::new(forward, near, far);

        self._render_frustum(&frustum, Some(color));
    }
}

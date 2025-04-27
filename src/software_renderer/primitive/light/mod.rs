use crate::{
    color,
    matrix::Mat4,
    render::Renderer,
    scene::{
        empty::EmptyDisplayKind,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
    },
    software_renderer::SoftwareRenderer,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
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

        // Renders a line connecting the light (position) to the ground plane.

        self.render_line(
            y_indicator_line_start,
            y_indicator_line_end,
            color::DARK_GRAY,
        );
    }

    fn render_light_indicator(&mut self, transform: &Mat4, intensities: Vec3) {
        let tone_mapped_intensities = self.get_tone_mapped_color_from_hdr(intensities);

        // Renders the light as a colored sphere, at its scene node's position.

        let scaled_transform = Mat4::scale(vec3::ONES * 0.1) * *transform;

        self.render_empty(
            &scaled_transform,
            EmptyDisplayKind::Sphere(8),
            false,
            Some(tone_mapped_intensities),
        );
    }

    pub(in crate::software_renderer) fn _render_ambient_light(
        &mut self,
        transform: &Mat4,
        light: &AmbientLight,
    ) {
        self.render_light_indicator(transform, light.intensities);

        let position = (Vec4::position(Default::default()) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);
    }

    pub(in crate::software_renderer) fn _render_directional_light(
        &mut self,
        transform: &Mat4,
        light: &DirectionalLight,
    ) {
        self.render_light_indicator(transform, light.intensities);

        let position = (Vec4::position(Default::default()) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        // Derive the light's orientation matrix using its direction vector.

        let forward = light.get_direction().to_vec3();
        let right = vec3::UP.cross(forward).as_normal();
        let up = forward.cross(right).as_normal();

        let scale = Mat4::scale_uniform(2.0);

        let rotation = Mat4::tbn(right, up, forward);

        let scale_rotation = scale * rotation;

        // Renders the light as several arrows pointing in the light direction,
        // clustered around the scene node's position.

        let tone_mapped_intensities = self.get_tone_mapped_color_from_hdr(light.intensities);

        // Draws 4 arrows, offset on the world X- and Z-axis.

        static ARROW_X_Z_OFFSETS: [(f32, f32); 4] =
            [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)];

        for (x_offset, z_offset) in ARROW_X_Z_OFFSETS {
            let translation = Mat4::translation(
                position
                    + Vec3 {
                        x: x_offset * 2.0,
                        z: z_offset * 2.0,
                        ..Default::default()
                    },
            );

            let transform = scale_rotation * translation;

            self.render_empty(
                &transform,
                EmptyDisplayKind::Arrow,
                false,
                Some(tone_mapped_intensities),
            );
        }
    }

    pub(in crate::software_renderer) fn _render_point_light(
        &mut self,
        transform: &Mat4,
        light: &PointLight,
    ) {
        self.render_light_indicator(transform, light.intensities);

        let position = (Vec4::position(Default::default()) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        // Renders the light as a colored sphere, at its position, surrounded by
        // a larger sphere to visualize the light's effective lighting radius.

        let tone_mapped_intensities = self.get_tone_mapped_color_from_hdr(light.intensities);

        let scaled_transform = Mat4::scale(vec3::ONES * light.influence_distance) * *transform;

        self.render_empty(
            &scaled_transform,
            EmptyDisplayKind::Sphere(16),
            false,
            Some(tone_mapped_intensities),
        );
    }

    pub(in crate::software_renderer) fn _render_spot_light(
        &mut self,
        transform: &Mat4,
        light: &SpotLight,
    ) {
        self.render_light_indicator(transform, light.intensities);

        let position = (Vec4::position(Default::default()) * (*transform)).to_vec3();

        self.render_light_ground_contact(&position);

        let tone_mapped_intensities = self.get_tone_mapped_color_from_hdr(light.intensities);

        // Renders a colored sphere at the light's position.

        let scaled_transform = Mat4::scale(vec3::ONES * 0.1) * *transform;

        self.render_empty(
            &scaled_transform,
            EmptyDisplayKind::Sphere(8),
            true,
            Some(tone_mapped_intensities),
        );

        // Derive the light's orientation matrix using existing basis vectors.

        let (forward, right, up) = (
            light.look_vector.get_forward(),
            light.look_vector.get_right(),
            light.look_vector.get_up(),
        );

        // Renders a line extending from the light's position to its maximum
        // effective lighting distance, in the lighting direction.

        let far_plane_center = position + forward * light.influence_distance;

        self.render_line(position, far_plane_center, color::WHITE);

        let rotation = Mat4::tbn(right, up, forward);

        // Renders a disk representing the light's outer cone, at a distance
        // approximating the light's effective influence on the scene.

        let inner_opposite_over_adjacent = light.get_inner_cutoff_angle().tan();
        let inner_radius = inner_opposite_over_adjacent * light.influence_distance;
        let inner_scale = Mat4::scale(vec3::ONES * inner_radius);
        let inner_transform = inner_scale * rotation * Mat4::translation(far_plane_center);

        self.render_empty(
            &inner_transform,
            EmptyDisplayKind::Circle(16),
            false,
            Some(tone_mapped_intensities),
        );

        // Renders a disk representing the light's inner cone, at a distance
        // approximating the light's effective influence on the scene.

        let outer_opposite_over_adjacent = light.get_outer_cutoff_angle().tan();
        let outer_radius = outer_opposite_over_adjacent * light.influence_distance;
        let outer_scale = Mat4::scale(vec3::ONES * outer_radius);
        let outer_transform = outer_scale * rotation * Mat4::translation(far_plane_center);

        // Connects the light position to the outer cone, as 4 connecting segments.

        self.render_empty(
            &outer_transform,
            EmptyDisplayKind::Circle(16),
            false,
            Some(tone_mapped_intensities),
        );

        self.render_line(
            position,
            (Vec4::position(vec3::UP) * outer_transform).to_vec3(),
            tone_mapped_intensities,
        );

        self.render_line(
            position,
            (Vec4::position(vec3::RIGHT) * outer_transform).to_vec3(),
            tone_mapped_intensities,
        );

        self.render_line(
            position,
            (Vec4::position(-vec3::UP) * outer_transform).to_vec3(),
            tone_mapped_intensities,
        );

        self.render_line(
            position,
            (Vec4::position(-vec3::RIGHT) * outer_transform).to_vec3(),
            tone_mapped_intensities,
        );
    }
}

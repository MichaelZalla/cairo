use crate::{
    color::{self, Color},
    graphics::Graphics,
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

fn get_color_for_intensities(intensities: &Vec3) -> Color {
    let mut c = *intensities;

    c = c.tone_map_exposure(1.0);

    c.linear_to_srgb();

    Color::from_vec3(c * 255.0)
}

fn projection_to_ndc(position_projection_space: Vec4) -> Vec3 {
    let w_inverse = 1.0 / position_projection_space.w;

    let mut position_ndc = position_projection_space.to_vec3();

    position_ndc *= w_inverse;

    position_ndc.x = (position_ndc.x + 1.0) / 2.0;
    position_ndc.y = (-position_ndc.y + 1.0) / 2.0;

    position_ndc
}

impl SoftwareRenderer {
    fn render_circle_at_ndc_space_position(
        &self,
        position_ndc_space: &Vec3,
        radius_ndc_space: f32,
        fill: Option<&Color>,
        border: Option<&Color>,
    ) {
        // Cull lines that are completely in front of our near plane (z1 <= 0
        // and z2 <= 0).

        if position_ndc_space.z <= 0.0 {
            return;
        }

        match self.framebuffer.as_ref() {
            Some(framebuffer_rc) => {
                let framebuffer = framebuffer_rc.borrow_mut();

                match &framebuffer.attachments.forward_ldr {
                    Some(forward_buffer_rc) => {
                        let mut forward_buffer = forward_buffer_rc.borrow_mut();

                        Graphics::circle(
                            &mut forward_buffer,
                            (position_ndc_space.x * self.viewport.width as f32) as u32,
                            (position_ndc_space.y * self.viewport.height as f32) as u32,
                            (radius_ndc_space * self.viewport.width as f32) as u32, fill, border);
                    },
                    None => panic!("Called SoftwareRenderer::render_circle_at_ndc_space_position() with no forward (LDR) framebuffer attachment!"),
                }
            }
            None => panic!(
                "Called SoftwareRenderer::render_circle_at_ndc_space_position() with no bound framebuffer!"
            ),
        }
    }

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
            &color::DARK_GRAY,
        );
    }

    fn render_light_circle(&mut self, position: &Vec3, radius: f32, intensities: &Vec3) {
        // Adds radius to position.x in camera-view space, before transforming
        // position to NDC space.

        let shader_context = self.shader_context.borrow();

        let (start_view_space, start_ndc_space) = {
            let view_space = Vec4::new(*position, 1.0) * shader_context.view_inverse_transform;

            let view_projection_space = view_space * shader_context.projection_transform;

            let ndc_space = projection_to_ndc(view_projection_space);

            (view_space, ndc_space)
        };

        let end_ndc_space = {
            let view_space = Vec4 {
                x: start_view_space.x + radius,
                y: start_view_space.y,
                z: start_view_space.z,
                w: start_view_space.w,
            };

            let view_projection_space = view_space * shader_context.projection_transform;

            projection_to_ndc(view_projection_space)
        };

        let border_color = get_color_for_intensities(intensities);

        let horizontal_radius_ndc = end_ndc_space.x - start_ndc_space.x;

        self.render_circle_at_ndc_space_position(
            &start_ndc_space,
            horizontal_radius_ndc,
            None,
            Some(&border_color),
        );
    }

    pub(in crate::software_renderer) fn _render_ambient_light(
        &mut self,
        transform: &Mat4,
        light: &AmbientLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_circle(&position, 1.0, &light.intensities);

        self.render_light_ground_contact(&position);
    }

    pub(in crate::software_renderer) fn _render_directional_light(
        &mut self,
        transform: &Mat4,
        light: &DirectionalLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_circle(&position, 1.0, &light.intensities);

        let color = get_color_for_intensities(&light.intensities);

        let (start, end) = (position, position + light.get_direction().to_vec3() * 10.0);

        self.render_line(start, end, &color);

        self.render_light_ground_contact(&position);
    }

    pub(in crate::software_renderer) fn _render_point_light(
        &mut self,
        transform: &Mat4,
        light: &PointLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        let radius = light.influence_distance;

        self.render_light_circle(&position, 1.0, &light.intensities);
        self.render_light_circle(&position, radius, &light.intensities);

        self.render_light_ground_contact(&position);
    }

    pub(in crate::software_renderer) fn _render_spot_light(
        &mut self,
        transform: &Mat4,
        light: &SpotLight,
    ) {
        let position = (Vec4::new(Default::default(), 1.0) * (*transform)).to_vec3();

        self.render_light_circle(&position, 1.0, &light.intensities);

        self.render_light_ground_contact(&position);

        let forward = light.look_vector.get_forward().as_normal();

        let target_position = position + forward * light.influence_distance;

        self.render_line(position, target_position, &color::WHITE);

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

use crate::{
    color::{self},
    pipeline::Pipeline,
    scene::camera::Camera,
    shader::{alpha::AlphaShader, fragment::FragmentShader, geometry::GeometryShader},
};

impl<'a, F, A, G> Pipeline<'a, F, A, G>
where
    F: FragmentShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
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
}

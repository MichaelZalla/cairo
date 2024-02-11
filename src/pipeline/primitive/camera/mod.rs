use crate::{
    color::{self},
    pipeline::Pipeline,
    scene::camera::Camera,
    shader::geometry::GeometryShader,
};

impl<'a, G> Pipeline<'a, G>
where
    G: GeometryShader<'a>,
{
    pub fn render_camera(&mut self, camera: &Camera) {
        let origin = camera.look_vector.get_position();

        // Target

        self.render_line(origin, camera.look_vector.get_target(), color::WHITE);

        let aspect_ratio = camera.get_aspect_ratio();

        let right_for_aspect_ratio = camera.look_vector.get_right() * aspect_ratio;

        // Top
        self.render_line(
            origin + camera.look_vector.get_up() - right_for_aspect_ratio,
            origin + camera.look_vector.get_up() + right_for_aspect_ratio,
            color::WHITE,
        );

        // Bottom
        self.render_line(
            origin - camera.look_vector.get_up() - right_for_aspect_ratio,
            origin - camera.look_vector.get_up() + right_for_aspect_ratio,
            color::WHITE,
        );

        // Left
        self.render_line(
            origin - right_for_aspect_ratio - camera.look_vector.get_up(),
            origin - right_for_aspect_ratio + camera.look_vector.get_up(),
            color::WHITE,
        );

        // Right
        self.render_line(
            origin + right_for_aspect_ratio - camera.look_vector.get_up(),
            origin + right_for_aspect_ratio + camera.look_vector.get_up(),
            color::WHITE,
        );
    }
}

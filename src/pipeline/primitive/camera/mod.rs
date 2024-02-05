use crate::{
    color::{self},
    pipeline::Pipeline,
    scene::camera::Camera,
};

impl<'a> Pipeline<'a> {
    pub fn render_camera(&mut self, camera: &Camera) {
        let origin = camera.look_vector.get_position();

        // Target

        self.render_line(origin, camera.look_vector.get_target(), color::WHITE);

        let aspect_ratio_option = camera.get_aspect_ratio();

        match aspect_ratio_option {
            Some(aspect_ratio) => {
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
            None => {
                todo!("Render an orthographic camera.")
            }
        }
    }
}

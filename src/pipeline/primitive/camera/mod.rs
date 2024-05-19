use crate::{color, pipeline::Pipeline, scene::camera::Camera};

impl<'a> Pipeline<'a> {
    pub fn render_camera(&mut self, camera: &Camera) {
        // World space view volume.

        let frustum = camera.get_world_space_frustum();

        // View volume

        self.render_frustum(&frustum, None);

        // Target

        self.render_line(
            (frustum.near[0] + frustum.near[2]) / 2.0,
            (frustum.far[0] + frustum.far[2]) / 2.0,
            color::WHITE,
        );

        self.render_point_indicator(Default::default(), 5.0);
    }
}

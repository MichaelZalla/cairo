use crate::{
    color::{self, Color},
    render::Renderer,
    scene::camera::Camera,
    software_renderer::SoftwareRenderer,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_camera(
        &mut self,
        camera: &Camera,
        color: Option<Color>,
    ) {
        // World space view volume.

        let frustum = camera.get_frustum();

        // View volume

        self.render_frustum(frustum, color);

        // Target

        self.render_line(
            (frustum.near[0] + frustum.near[2]) / 2.0,
            (frustum.far[0] + frustum.far[2]) / 2.0,
            color::WHITE,
        );

        self.render_point_indicator(Default::default(), 5.0);
    }
}

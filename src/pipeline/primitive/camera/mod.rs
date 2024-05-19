use crate::{color, pipeline::Pipeline, scene::camera::Camera};

impl<'a> Pipeline<'a> {
    pub fn render_camera(&mut self, camera: &Camera) {
        // World space view volume.

        let (near_plane_points_world_space, far_plane_points_world_space) =
            camera.get_world_space_frustum();

        // View volume

        self.render_frustum(
            near_plane_points_world_space,
            far_plane_points_world_space,
            None,
        );

        // Target

        self.render_line(
            (near_plane_points_world_space[0] + near_plane_points_world_space[2]) / 2.0,
            (far_plane_points_world_space[0] + far_plane_points_world_space[2]) / 2.0,
            color::WHITE,
        );
    }
}

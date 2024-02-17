use std::f32::consts::PI;

use crate::{
    color,
    pipeline::Pipeline,
    scene::camera::{Camera, CameraProjectionKind},
    vec::vec4::Vec4,
};

impl<'a> Pipeline<'a> {
    pub fn render_camera(&mut self, camera: &Camera) {
        // Canonical (clip space) view volume.

        let front_top_left_clip_space = Vec4 {
            x: -1.0,
            y: 1.0,
            z: camera.get_projection_z_near(),
            w: 1.0,
        };

        let front_top_right_clip_space = Vec4 {
            x: 1.0,
            ..front_top_left_clip_space
        };

        let front_bottom_left_clip_space = Vec4 {
            y: -1.0,
            ..front_top_left_clip_space
        };

        let front_bottom_right_clip_space = Vec4 {
            x: 1.0,
            ..front_bottom_left_clip_space
        };

        let near_plane_points_clip_space = [
            front_top_left_clip_space,
            front_top_right_clip_space,
            front_bottom_right_clip_space,
            front_bottom_left_clip_space,
        ];

        let far_plane_points_clip_space = near_plane_points_clip_space.clone().map(|mut coord| {
            coord.z = camera.get_projection_z_far();
            coord
        });

        // World space view volume.

        let near_plane_points_world_space;
        let far_plane_points_world_space;

        match camera.get_kind() {
            CameraProjectionKind::Perspective => {
                let fov = camera.get_field_of_view().unwrap();
                let fov_rad = fov * PI / 180.0;

                let opposite_over_adjacent_x = (fov_rad / 2.0).tan();

                let opposite_over_adjacent_y =
                    (fov_rad / 2.0).tan() / camera.get_aspect_ratio().unwrap();

                near_plane_points_world_space = near_plane_points_clip_space
                    .map(|mut coord| {
                        coord.x *= camera.get_projection_z_near() * opposite_over_adjacent_x;
                        coord.y *= camera.get_projection_z_near() * opposite_over_adjacent_y;

                        let coord_projection_space = coord * camera.get_view_transform();

                        coord_projection_space
                    })
                    .map(|coord| coord.to_vec3());

                far_plane_points_world_space = far_plane_points_clip_space
                    .map(|mut coord| {
                        coord.x *= camera.get_projection_z_far() * opposite_over_adjacent_x;
                        coord.y *= camera.get_projection_z_far() * opposite_over_adjacent_y;

                        let coord_projection_space = coord * camera.get_view_transform();

                        coord_projection_space
                    })
                    .map(|coord| coord.to_vec3());
            }
            CameraProjectionKind::Orthographic => {
                near_plane_points_world_space = near_plane_points_clip_space
                    .map(|coord| {
                        let coord_projection_space =
                            coord * camera.get_projection_inverse() * camera.get_view_transform();

                        coord_projection_space
                    })
                    .map(|coord| coord.to_vec3());

                far_plane_points_world_space = far_plane_points_clip_space
                    .map(|coord| {
                        let coord_projection_space =
                            coord * camera.get_projection_inverse() * camera.get_view_transform();

                        coord_projection_space
                    })
                    .map(|coord| coord.to_vec3());
            }
        }

        // View volume

        self.render_frustum(near_plane_points_world_space, far_plane_points_world_space);

        // Target

        self.render_line(
            (near_plane_points_world_space[0] + near_plane_points_world_space[2]) / 2.0,
            (far_plane_points_world_space[0] + far_plane_points_world_space[2]) / 2.0,
            color::WHITE,
        );
    }
}

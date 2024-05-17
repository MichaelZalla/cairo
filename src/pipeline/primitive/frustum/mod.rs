use crate::{
    color::{self, Color},
    pipeline::Pipeline,
    vec::vec3::Vec3,
};

impl<'a> Pipeline<'a> {
    pub fn render_frustum(
        &mut self,
        near_plane_points_world_space: [Vec3; 4],
        far_plane_points_world_space: [Vec3; 4],
        color: Option<Color>,
    ) {
        // Draw near plane (red).

        for (index, _point) in near_plane_points_world_space.as_slice().iter().enumerate() {
            self.render_line(
                near_plane_points_world_space[index],
                near_plane_points_world_space[if index == 3 { 0 } else { index + 1 }],
                match color {
                    Some(color) => color,
                    None => color::RED,
                },
            );
        }

        // Draw far plane (blue).

        for (index, _point) in far_plane_points_world_space.as_slice().iter().enumerate() {
            self.render_line(
                far_plane_points_world_space[index],
                far_plane_points_world_space[if index == 3 { 0 } else { index + 1 }],
                match color {
                    Some(color) => color,
                    None => color::BLUE,
                },
            );
        }

        // Connect the 2 planes.

        for i in 0..4 {
            self.render_line(
                near_plane_points_world_space[i],
                far_plane_points_world_space[i],
                match color {
                    Some(color) => color,
                    None => color::YELLOW,
                },
            );
        }
    }
}

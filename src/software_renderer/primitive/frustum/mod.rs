use crate::{
    color::{self, Color},
    render::Renderer,
    scene::camera::frustum::Frustum,
    software_renderer::SoftwareRenderer,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_frustum(
        &mut self,
        frustum: &Frustum,
        color: Option<Color>,
    ) {
        // Draw near plane (red).

        for (index, _point) in frustum.near.as_slice().iter().enumerate() {
            self.render_line(
                frustum.near[index],
                frustum.near[if index == 3 { 0 } else { index + 1 }],
                match color {
                    Some(color) => color,
                    None => color::RED,
                },
            );
        }

        // Draw far plane (blue).

        for (index, _point) in frustum.far.as_slice().iter().enumerate() {
            self.render_line(
                frustum.far[index],
                frustum.far[if index == 3 { 0 } else { index + 1 }],
                match color {
                    Some(color) => color,
                    None => color::BLUE,
                },
            );
        }

        // Connect the 2 planes.

        for i in 0..4 {
            self.render_line(
                frustum.near[i],
                frustum.far[i],
                match color {
                    Some(color) => color,
                    None => color::YELLOW,
                },
            );
        }
    }
}

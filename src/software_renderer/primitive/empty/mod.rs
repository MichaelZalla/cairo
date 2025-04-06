use std::f32::consts::TAU;

use crate::{
    color, matrix::Mat4, render::Renderer, scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer, vec::vec4::Vec4,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_empty(
        &mut self,
        transform: &Mat4,
        display_kind: EmptyDisplayKind,
    ) {
        match display_kind {
            EmptyDisplayKind::Axes => {
                let world_position = (Vec4::new(Default::default(), 1.0) * *transform).to_vec3();

                self.render_axes(Some(world_position), None);
            }
            EmptyDisplayKind::Circle(divisions) => {
                // 1. Defines a unit circle as a set of points (in object space).

                let arc_length = TAU / divisions as f32;

                let mut points = vec![Vec4::new(Default::default(), 1.0); divisions];

                for (index, point) in points.iter_mut().enumerate() {
                    let theta = arc_length * index as f32;

                    point.x = theta.cos();
                    point.y = theta.sin();
                }

                // 2. Transforms the points into world space, based on `transform`.

                for point in points.iter_mut() {
                    *point *= *transform;
                }

                // 3. Renders the transformed circle as a set of connected line segments;

                for i in 0..points.len() {
                    let start = &points[i];

                    let end = &points[if i == points.len() - 1 { 0 } else { i + 1 }];

                    self.render_line(start.to_vec3(), end.to_vec3(), color::WHITE);
                }
            }
        }
    }
}

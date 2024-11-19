use crate::{color::Color, geometry::primitives::ray::Ray, software_renderer::SoftwareRenderer};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_ray(&mut self, ray: &Ray, color: Color) {
        self._render_circle(&ray.origin, 0.2, color);

        if ray.t < f32::MAX {
            let start = ray.origin + ray.direction * 0.2;
            let end = ray.origin + ray.direction * (ray.t - 0.2);

            self._render_line(start, end, color);
        }
    }
}

use cairo::{
    color,
    geometry::primitives::plane::Plane,
    render::Renderer,
    software_renderer::SoftwareRenderer,
    vec::vec3::{self, Vec3},
};

#[derive(Default, Debug, Clone)]
pub struct PlaneCollider {
    pub point: Vec3,
    pub plane: Plane,
}

impl PlaneCollider {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        let plane = Plane::new(point, direction);

        Self { point, plane }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        let normal = self.plane.normal;

        let mut right = normal.cross(vec3::UP);

        if right.mag() < f32::EPSILON {
            right = normal.cross(vec3::FORWARD);
        }

        right = right.as_normal();

        let up = normal.cross(-right);

        // Normal
        renderer.render_line(self.point, self.point + normal, color::BLUE);

        // Tangent
        renderer.render_line(self.point, self.point + right, color::RED);

        // Bitangent
        renderer.render_line(self.point, self.point + up, color::GREEN);
    }
}

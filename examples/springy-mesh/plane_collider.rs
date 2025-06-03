use cairo::{
    geometry::primitives::plane::Plane,
    matrix::Mat4,
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
        let up = self.plane.normal;

        let mut forward = up.cross(vec3::UP);

        if forward.mag() < f32::EPSILON {
            forward = up.cross(vec3::FORWARD);
        }

        forward = forward.as_normal();

        let right = up.cross(forward);

        let tbn = Mat4::translation(self.point) * Mat4::tbn(right, up, forward);

        renderer.render_ground_plane(30, Some(&tbn));
    }
}

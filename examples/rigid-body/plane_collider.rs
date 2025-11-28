use cairo::{
    color,
    geometry::primitives::plane::Plane,
    matrix::Mat4,
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Clone)]
pub struct PlaneCollider {
    pub point: Vec3,
    pub plane: Plane,
}

impl Default for PlaneCollider {
    fn default() -> Self {
        let point = Vec3::default();

        Self {
            point,
            plane: Plane::new(point, vec3::UP),
        }
    }
}

impl PlaneCollider {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        let plane = Plane::new(point, direction);

        Self { point, plane }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        // Calculate the coordinate frame for the plane
        let (_forward, right, up) = self.plane.normal.basis();

        // Render the plane as an orange square
        let scale = Mat4::scale_uniform(20.0);
        let translate = Mat4::translation(self.point);
        let rotate = Mat4::tbn(right, up, self.plane.normal);

        let transform = scale * rotate * translate;

        renderer.render_empty(
            &transform,
            EmptyDisplayKind::Square,
            false,
            Some(color::ORANGE),
        );

        // Render the normal vector as a green line
        renderer.render_line(
            self.point,
            self.point + self.plane.normal * 5.0,
            color::GREEN,
        );
    }
}

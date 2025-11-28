use cairo::{
    color, matrix::Mat4, physics::simulation::collision::colliders::plane_collider::PlaneCollider,
    render::Renderer, scene::empty::EmptyDisplayKind, software_renderer::SoftwareRenderer,
};

// Springy-mesh specific extension trait for PlaneCollider rendering
pub trait Renderable {
    fn render(&self, renderer: &mut SoftwareRenderer);
}

impl Renderable for PlaneCollider {
    fn render(&self, renderer: &mut SoftwareRenderer) {
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

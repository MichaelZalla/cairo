use crate::{
    color::{self, Color},
    geometry::{accelerator::static_triangle_bvh::StaticTriangleBVH, primitives::aabb::AABB},
    matrix::Mat4,
    render::Renderer,
    software_renderer::SoftwareRenderer,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_aabb(
        &mut self,
        aabb: &AABB,
        world_transform: &Mat4,
        color: Color,
    ) {
        let mut vertices = aabb.get_vertices();

        for v in vertices.iter_mut() {
            *v *= *world_transform;
        }

        // Near plane.

        self.render_line_loop(&vertices, 0, 3, color);

        // Far plane.

        self.render_line_loop(&vertices, 4, 7, color);

        // Connect near and far planes.

        for i in 0..4 {
            self.render_line(vertices[i], vertices[i + 4], color);
        }
    }

    pub fn render_bvh(&mut self, bvh: &StaticTriangleBVH) {
        let current_world_transform = Mat4::identity();

        // Render the BVH root's AABB.

        let root = &bvh.root;

        self.render_aabb(&root.aabb, &current_world_transform, color::GREEN);
    }
}

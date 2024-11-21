use crate::{
    color::{hsv_to_rgb, Color},
    geometry::{accelerator::static_triangle_bvh::StaticTriangleBVH, primitives::aabb::AABB},
    matrix::Mat4,
    render::Renderer,
    software_renderer::SoftwareRenderer,
    vec::vec3::Vec3,
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

    pub fn render_bvh(&mut self, bvh: &StaticTriangleBVH, maximum_depth: u8) {
        let current_world_transform = Mat4::identity();

        for node_index in 0..bvh.nodes_used {
            let node = &bvh.nodes[node_index];

            if node.depth > maximum_depth {
                continue;
            }

            static BVH_MAX_DEPTH: f32 = 16.0;
            static BVH_DEPTH_ALPHA_STEP: f32 = 1.0 / BVH_MAX_DEPTH;

            let h = 360.0 * (BVH_DEPTH_ALPHA_STEP * node.depth as f32).min(1.0);
            let s = 1.0;
            let v = 0.5;

            let hsv = Vec3 { x: h, y: s, z: v };

            let rgb = hsv_to_rgb(hsv);

            self._render_aabb(
                &node.aabb,
                &current_world_transform,
                Color::from_vec3(rgb * 255.0),
            );
        }
    }
}

use cairo::{
    animation::lerp,
    geometry::{
        accelerator::static_triangle_bvh::StaticTriangleBVH,
        intersect::test_aabb_aabb,
        primitives::{aabb::AABB, line_segment::LineSegment, triangle::Triangle},
    },
};

fn intersect_line_segment_triangle(
    segment: &mut LineSegment,
    tri_index: usize,
    triangle: &Triangle,
) {
    let (p, q) = (segment.start, segment.end);

    // Compute the distance of P to the triangle's normal-facing plane.

    let p_distance = triangle.plane.get_signed_distance(&p);

    // Exit if start point P is behind the plane.

    if p_distance < 0.0 {
        return;
    }

    // Compute the distance of Q to the triangle's normal-facing plane.

    let q_distance = triangle.plane.get_signed_distance(&q);

    // Exit if end point Q is in front of the plane.

    if q_distance >= 0.0 {
        return;
    }

    // Compute the point-of-intersection S of the line PQ with the plane.

    let total_distance = p_distance - q_distance;

    let t = p_distance / total_distance;

    let s = lerp(p, q, t);

    // Compute the barycentric coordinate U; exit if outside the [0..1] range.

    let u = s.dot(triangle.edge_plane_bc.normal) - triangle.edge_plane_bc.d;

    if !(0.0..=1.0).contains(&u) {
        return;
    }

    // Compute the barycentric coordinate V; exit if negative.

    let v = s.dot(triangle.edge_plane_ca.normal) - triangle.edge_plane_ca.d;

    if v < 0.0 {
        return;
    }

    // Compute the barycentric coordinate W; exit if negative.

    let w = 1.0 - u - v;

    if w < 0.0 {
        return;
    }

    // Segment PQ intersects triangle.

    if t < segment.t {
        segment.t = t;
        segment.colliding_primitive.replace(tri_index);
    }
}

pub fn intersect_line_segment_bvh(segment: &mut LineSegment, bvh: &StaticTriangleBVH) {
    let mut segment_aabb = AABB::default();

    segment_aabb.grow(&segment.start);
    segment_aabb.grow(&segment.end);

    intersect_line_segment_bvh_node(segment, &segment_aabb, bvh, 0)
}

fn intersect_line_segment_bvh_node(
    segment: &mut LineSegment,
    segment_aabb: &AABB,
    bvh: &StaticTriangleBVH,
    node_index: usize,
) {
    let node = &bvh.nodes[node_index];

    if !test_aabb_aabb(segment_aabb, &node.aabb) {
        return;
    };

    if node.is_leaf() {
        let start = node.primitives_start_index as usize;
        let end = start + node.primitives_count as usize;

        for tri_index_index in start..end {
            let tri_index = bvh.tri_indices[tri_index_index];

            let triangle = &bvh.tris[tri_index];

            intersect_line_segment_triangle(segment, tri_index, triangle);
        }
    } else {
        intersect_line_segment_bvh_node(segment, segment_aabb, bvh, node.left_child_index as usize);
        intersect_line_segment_bvh_node(
            segment,
            segment_aabb,
            bvh,
            node.left_child_index as usize + 1,
        );
    }
}

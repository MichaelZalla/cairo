use cairo::{
    animation::lerp,
    geometry::{
        accelerator::{
            static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
            static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::test_aabb_aabb,
        primitives::{aabb::AABB, line_segment::LineSegment, triangle::Triangle},
    },
    vec::vec4::Vec4,
};

fn intersect_line_segment_triangle(
    segment: &mut LineSegment,
    bvh_instance_index: usize,
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

        segment.colliding_bvh_index.replace(bvh_instance_index);

        segment.colliding_primitive.replace(tri_index);
    }
}

pub fn intersect_line_segment_tlas(segment: &mut LineSegment, tlas: &StaticTriangleTLAS) {
    let mut segment_aabb = AABB::default();

    segment_aabb.grow(&segment.start);
    segment_aabb.grow(&segment.end);

    intersect_line_segment_tlas_node(segment, &segment_aabb, tlas, 0)
}

pub fn intersect_line_segment_tlas_node(
    segment: &mut LineSegment,
    segment_aabb: &AABB,
    tlas: &StaticTriangleTLAS,
    node_index: usize,
) {
    let node = &tlas.nodes[node_index];

    if node.is_leaf {
        let bvh_instance_index = node.bvh_instance_index as usize;

        let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

        intersect_line_segment_bvh(segment, bvh_instance_index, bvh_instance);

        return;
    }

    let left_child_index = node.left_child_index as usize;
    let right_child_index = left_child_index + 1;

    let left_aabb = &tlas.nodes[left_child_index].aabb;

    if test_aabb_aabb(segment_aabb, left_aabb) {
        intersect_line_segment_tlas_node(segment, segment_aabb, tlas, left_child_index);
    }

    if right_child_index > 0 {
        let right_aabb = &tlas.nodes[right_child_index].aabb;

        if test_aabb_aabb(segment_aabb, right_aabb) {
            intersect_line_segment_tlas_node(segment, segment_aabb, tlas, right_child_index);
        }
    }
}

fn intersect_line_segment_bvh(
    segment: &mut LineSegment,
    bvh_instance_index: usize,
    bvh_instance: &StaticTriangleBVHInstance,
) {
    let mut transformed_segment = *segment;

    transformed_segment.start =
        (Vec4::new(transformed_segment.start, 1.0) * bvh_instance.inverse_transform).to_vec3();

    transformed_segment.end =
        (Vec4::new(transformed_segment.end, 1.0) * bvh_instance.inverse_transform).to_vec3();

    let mut transformed_segment_aabb = AABB::default();

    transformed_segment_aabb.grow(&transformed_segment.start);
    transformed_segment_aabb.grow(&transformed_segment.end);

    intersect_line_segment_bvh_node(
        &mut transformed_segment,
        &transformed_segment_aabb,
        bvh_instance_index,
        &bvh_instance.bvh,
        0,
    );

    if let (Some(colliding_bvh_index), Some(colliding_primitive)) = (
        transformed_segment.colliding_bvh_index,
        transformed_segment.colliding_primitive,
    ) {
        segment.t = transformed_segment.t;

        segment.transformed_length = ((segment.end - segment.start) * bvh_instance.transform)
            .mag()
            .abs();

        segment.colliding_bvh_index.replace(colliding_bvh_index);

        segment.colliding_primitive.replace(colliding_primitive);
    }
}

fn intersect_line_segment_bvh_node(
    segment: &mut LineSegment,
    segment_aabb: &AABB,
    bvh_instance_index: usize,
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

            intersect_line_segment_triangle(segment, bvh_instance_index, tri_index, triangle);
        }
    } else {
        let (left, right) = (
            node.left_child_index as usize,
            node.left_child_index as usize + 1,
        );

        intersect_line_segment_bvh_node(segment, segment_aabb, bvh_instance_index, bvh, left);
        intersect_line_segment_bvh_node(segment, segment_aabb, bvh_instance_index, bvh, right);
    }
}

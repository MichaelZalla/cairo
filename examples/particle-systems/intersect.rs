use cairo::{
    geometry::{
        accelerator::{
            static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
            static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::test_aabb_aabb,
        primitives::{MedialPrimitive, aabb::AABB},
    },
    vec::vec4::Vec4,
};

// Generic BVH traversal for any primitive with a medial axis

pub fn intersect_medial_primitive_tlas<P: MedialPrimitive>(
    primitive: &mut P,
    tlas: &StaticTriangleTLAS,
) {
    let primitive_aabb = primitive.aabb();

    intersect_medial_primitive_tlas_node(primitive, &primitive_aabb, tlas, 0)
}

pub fn intersect_medial_primitive_tlas_node<P: MedialPrimitive>(
    primitive: &mut P,
    primitive_aabb: &AABB,
    tlas: &StaticTriangleTLAS,
    node_index: usize,
) {
    let node = &tlas.nodes[node_index];

    if node.is_leaf() {
        let bvh_instance_index = node.bvh_instance_index as usize;

        let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

        intersect_medial_primitive_bvh(primitive, bvh_instance_index, bvh_instance);

        return;
    }

    let left_child_index = node.left_child_index as usize;
    let right_child_index = node.right_child_index as usize;

    let left_aabb = &tlas.nodes[left_child_index].aabb;

    if test_aabb_aabb(primitive_aabb, left_aabb) {
        intersect_medial_primitive_tlas_node(primitive, primitive_aabb, tlas, left_child_index);
    }

    if right_child_index > 0 {
        let right_aabb = &tlas.nodes[right_child_index].aabb;

        if test_aabb_aabb(primitive_aabb, right_aabb) {
            intersect_medial_primitive_tlas_node(
                primitive,
                primitive_aabb,
                tlas,
                right_child_index,
            );
        }
    }
}

pub fn intersect_medial_primitive_bvh<P: MedialPrimitive>(
    primitive: &mut P,
    bvh_instance_index: usize,
    bvh_instance: &StaticTriangleBVHInstance,
) {
    let mut transformed_primitive = *primitive;

    // Transform the medial segment into BVH local space
    let segment = transformed_primitive.medial_segment_mut();

    segment.start = (Vec4::position(segment.start) * bvh_instance.inverse_transform).to_vec3();
    segment.end = (Vec4::position(segment.end) * bvh_instance.inverse_transform).to_vec3();

    let transformed_primitive_aabb = transformed_primitive.aabb();

    intersect_medial_primitive_bvh_node(
        &mut transformed_primitive,
        &transformed_primitive_aabb,
        bvh_instance_index,
        &bvh_instance.bvh,
        0,
    );

    // Copy collision results back to original primitive
    let transformed_segment = transformed_primitive.medial_segment();

    if let (Some(colliding_bvh_index), Some(colliding_primitive)) = (
        transformed_segment.colliding_bvh_index,
        transformed_segment.colliding_primitive,
    ) {
        let original_segment = primitive.medial_segment_mut();

        original_segment.t = transformed_segment.t;

        original_segment.transformed_length = ((original_segment.end - original_segment.start)
            * bvh_instance.transform)
            .mag()
            .abs();

        original_segment
            .colliding_bvh_index
            .replace(colliding_bvh_index);

        original_segment
            .colliding_primitive
            .replace(colliding_primitive);
    }
}

fn intersect_medial_primitive_bvh_node<P: MedialPrimitive>(
    primitive: &mut P,
    primitive_aabb: &AABB,
    bvh_instance_index: usize,
    bvh: &StaticTriangleBVH,
    node_index: usize,
) {
    let node = &bvh.nodes[node_index];

    if !test_aabb_aabb(primitive_aabb, &node.aabb) {
        return;
    };

    if node.is_leaf() {
        let start = node.primitives_start_index as usize;
        let end = start + node.primitives_count as usize;

        for tri_index_index in start..end {
            let tri_index = bvh.tri_indices[tri_index_index];

            let triangle = &bvh.tris[tri_index];

            if let Some(_barycentric) = primitive.intersect_triangle(triangle) {
                let segment = primitive.medial_segment_mut();

                segment.colliding_bvh_index.replace(bvh_instance_index);
                segment.colliding_primitive.replace(tri_index);
            }
        }
    } else {
        let (left, right) = (
            node.left_child_index as usize,
            node.left_child_index as usize + 1,
        );

        intersect_medial_primitive_bvh_node(
            primitive,
            primitive_aabb,
            bvh_instance_index,
            bvh,
            left,
        );

        intersect_medial_primitive_bvh_node(
            primitive,
            primitive_aabb,
            bvh_instance_index,
            bvh,
            right,
        );
    }
}

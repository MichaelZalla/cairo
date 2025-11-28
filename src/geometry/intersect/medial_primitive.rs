//! Generic intersection routines for primitives with medial axes.
//!
//! This module provides efficient BVH and TLAS traversal for any primitive
//! that implements the `MedialPrimitive` trait (e.g., capsules, cylinders).

use crate::{
    geometry::{
        accelerator::{
            static_triangle_bvh::{
                StaticBVHCollision, StaticTriangleBVH, StaticTriangleBVHInstance,
            },
            static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::test_aabb_aabb,
        primitives::{MedialPrimitive, aabb::AABB},
    },
    vec::vec4::Vec4,
};

/// Intersects a medial primitive against a TLAS (top-level acceleration structure).
///
/// This function tests the primitive against all triangle meshes in the scene,
/// using the TLAS hierarchy for efficient broad-phase culling.
pub fn intersect_medial_primitive_tlas<P: MedialPrimitive>(
    primitive: &mut P,
    tlas: &StaticTriangleTLAS,
) {
    let primitive_aabb = primitive.aabb();

    intersect_medial_primitive_tlas_node(primitive, &primitive_aabb, tlas, 0)
}

/// Recursively traverses a TLAS node, testing the primitive against child nodes.
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

/// Intersects a medial primitive against a BVH instance.
///
/// This function transforms the primitive into the BVH's local space,
/// performs the intersection test, and transforms the results back.
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

    // Copy all collisions from the transformed primitive to the original
    let original_segment = primitive.medial_segment_mut();

    original_segment.transformed_length = ((original_segment.end - original_segment.start)
        * bvh_instance.transform)
        .mag()
        .abs();

    original_segment.collisions = transformed_segment.collisions;
}

/// Recursively traverses a BVH node, testing the primitive against triangles.
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

            if let Some(intersection) = primitive.intersect_triangle(triangle) {
                let segment = primitive.medial_segment_mut();

                // Create a collision with all the information
                let collision = StaticBVHCollision {
                    t: intersection.t,
                    colliding_bvh_index: bvh_instance_index,
                    colliding_primitive: tri_index,
                    barycentric: intersection.barycentric,
                };

                // Try to add it to the collision list
                segment.try_push_collision(collision);
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

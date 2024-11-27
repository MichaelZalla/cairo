use crate::geometry::primitives::aabb::AABB;

use super::static_triangle_bvh::StaticTriangleBVHInstance;

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticTriangleTLASNode {
    pub aabb: AABB,
    pub left_child_index: u32,
    pub right_child_index: u32,
    pub bvh_instance_index: u32,
}

impl StaticTriangleTLASNode {
    pub fn is_leaf(&self) -> bool {
        self.left_child_index == 0 && self.right_child_index == 0
    }
}

#[derive(Default, Debug, Clone)]
pub struct StaticTriangleTLAS {
    pub nodes: Vec<StaticTriangleTLASNode>,
    pub nodes_used: u32,
    pub bvh_instances: Vec<StaticTriangleBVHInstance>,
}

impl StaticTriangleTLAS {
    pub fn new(bvh_instances: Vec<StaticTriangleBVHInstance>) -> Self {
        let num_bvh_instances = bvh_instances.len() as u32;

        let max_node_count = num_bvh_instances * 2 - 1 + 1;

        let mut nodes = vec![StaticTriangleTLASNode::default(); max_node_count as usize];

        // Reserves a spot for the root (added last).

        let mut nodes_used = 1_u32;

        // Construct leaf nodes from the set of BVH instances.

        debug_assert!(num_bvh_instances <= 256);

        let mut work_queue: [u32; 256] = [0; 256];
        let mut work_queue_len = num_bvh_instances;

        for (bvh_instance_index, bvh_instance) in bvh_instances.iter().enumerate() {
            let node = &mut nodes[nodes_used as usize];

            work_queue[bvh_instance_index] = nodes_used;

            node.aabb.min = bvh_instance.world_aabb.min;
            node.aabb.max = bvh_instance.world_aabb.max;
            node.bvh_instance_index = bvh_instance_index as u32;

            node.left_child_index = 0;
            node.right_child_index = 0;

            nodes_used += 1;
        }

        // Use agglomerative clustering to pair together leaf nodes, using the
        // surface area heuristic.

        // See: https://www.graphics.cornell.edu/~bjw/IRT08Agglomerative.pdf

        let mut a = 0;

        let (mut b, _ab_aabb) = find_bvh_instance_pair_with_minimum_area(
            &nodes,
            work_queue.as_slice(),
            work_queue_len,
            a,
        );

        loop {
            if work_queue_len <= 1 {
                break;
            }

            let (c, bc_aabb) = find_bvh_instance_pair_with_minimum_area(
                &nodes,
                work_queue.as_slice(),
                work_queue_len,
                b,
            );

            if a == c {
                // Best pair-wise match in the work queue.
                // Bundle A and B together in a new internal node.

                let left_tlas_leaf_node_index = work_queue[a as usize];
                let right_tlas_leaf_node_index = work_queue[b as usize];

                let internal_node = &mut nodes[nodes_used as usize];

                internal_node.aabb = bc_aabb;
                internal_node.left_child_index = left_tlas_leaf_node_index;
                internal_node.right_child_index = right_tlas_leaf_node_index;

                // Reduce our work queue:

                // 1. Replace the node index at work_queue[a] with the index of
                // the new internal node.

                work_queue[a as usize] = nodes_used;

                nodes_used += 1;

                // 2. Swap the node index at B to the back of the queue, and
                //    decrement our queue size.

                // (No need to move `work_queue[b]`, just overwrite it's value).

                work_queue[b as usize] = work_queue[work_queue_len as usize - 1];

                work_queue_len -= 1;

                // Find our new B.

                b = if work_queue_len > 1 {
                    let (new_b, _) = find_bvh_instance_pair_with_minimum_area(
                        &nodes,
                        work_queue.as_slice(),
                        work_queue_len,
                        a,
                    );

                    new_b
                } else {
                    a
                };
            } else {
                a = b;
                b = c;
            }
        }

        let root_node_index = work_queue[a as usize] as usize;

        nodes[0] = nodes[root_node_index];

        Self {
            bvh_instances,
            nodes,
            nodes_used,
        }
    }
}

fn find_bvh_instance_pair_with_minimum_area(
    tlas_nodes: &[StaticTriangleTLASNode],
    work_queue: &[u32],
    work_queue_len: u32,
    bvh_instance_index: u32,
) -> (u32, AABB) {
    // Linear-time search for a pair of BVH instances (A, B) that, together,
    // forms the minimum AABB (by surface area).

    let a_index = bvh_instance_index;

    let a_tlas_node_index = work_queue[a_index as usize] as usize;

    let a_tlas_node = &tlas_nodes[a_tlas_node_index];

    let mut minimum_area = f32::MAX;

    let mut best_candidate_index = -1_isize;

    for b_index in 0..work_queue_len {
        // Don't compare A with A.

        if b_index != a_index {
            let b_tlas_node_index = work_queue[b_index as usize] as usize;

            let b_tlas_node = &tlas_nodes[b_tlas_node_index];

            // Compute the box bounding A and B.

            let mut ab_aabb = AABB::default();

            ab_aabb.grow_aabb(&a_tlas_node.aabb);
            ab_aabb.grow_aabb(&b_tlas_node.aabb);

            // Technically, half-area.

            let area = ab_aabb.extent().half_area_of_extent();

            // Compare the box's half-area to the minimum area seen so far.

            if area < minimum_area {
                minimum_area = area;

                best_candidate_index = b_index as isize;
            }
        }
    }

    assert!(best_candidate_index != -1);

    let b_tlas_node = &tlas_nodes[work_queue[best_candidate_index as usize] as usize];

    let aabb = {
        let mut aabb = AABB::default();

        aabb.grow_aabb(&a_tlas_node.aabb);
        aabb.grow_aabb(&b_tlas_node.aabb);

        aabb
    };

    (best_candidate_index as u32, aabb)
}

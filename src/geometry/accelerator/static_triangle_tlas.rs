use crate::geometry::primitives::aabb::AABB;

use super::static_triangle_bvh::StaticTriangleBVHInstance;

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticTriangleTLASNode {
    pub aabb: AABB,
    pub is_leaf: bool,
    pub left_child_index: u32,
    pub bvh_instance_index: u32,
}

#[derive(Default, Debug, Clone)]
pub struct StaticTriangleTLAS {
    pub nodes: Vec<StaticTriangleTLASNode>,
    pub nodes_used: usize,
    pub bvh_instances: Vec<StaticTriangleBVHInstance>,
}

impl StaticTriangleTLAS {
    pub fn new(bvh_instances: Vec<StaticTriangleBVHInstance>) -> Self {
        let num_bvh_instances = bvh_instances.len();

        let max_node_count = num_bvh_instances * 2 - 1;

        let mut nodes = vec![StaticTriangleTLASNode::default(); max_node_count];

        {
            let left_left_child = &mut nodes[3];

            left_left_child.is_leaf = true;
            left_left_child.bvh_instance_index = 0;
            left_left_child.aabb = bvh_instances[0].world_aabb;
        }

        {
            let left_right_child = &mut nodes[4];

            left_right_child.is_leaf = true;
            left_right_child.bvh_instance_index = 1;
            left_right_child.aabb = bvh_instances[1].world_aabb;
        }

        {
            let left_child_aabb = nodes[3].aabb;
            let right_child_aabb = nodes[4].aabb;

            let left_child = &mut nodes[1];

            left_child.left_child_index = 3;
            left_child.aabb.grow_aabb(&left_child_aabb);
            left_child.aabb.grow_aabb(&right_child_aabb);
        }

        {
            let right_child = &mut nodes[2];

            right_child.is_leaf = true;
            right_child.bvh_instance_index = 2;
            right_child.aabb = bvh_instances[2].world_aabb;
        }

        let left_aabb = nodes[1].aabb;
        let right_aabb = nodes[2].aabb;

        {
            let root = &mut nodes[0];

            root.left_child_index = 1;
            root.aabb.grow_aabb(&left_aabb);
            root.aabb.grow_aabb(&right_aabb);
        }

        Self {
            bvh_instances,
            nodes,
            nodes_used: 5,
        }
    }
}

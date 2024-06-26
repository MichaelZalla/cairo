use std::fmt::Display;

use crate::vec::vec3::Vec3;

use super::aabb::AABB;

#[derive(Default, Clone)]
pub struct OctTreeNode<T> {
    pub depth: usize,
    pub bounds: AABB,
    pub data_capacity: usize,
    pub data: Vec<T>,
    pub children: Vec<OctTreeNode<T>>,
}

impl<T: Copy + Display> OctTreeNode<T> {
    pub fn new(parent: &Self, bounds: AABB) -> Self {
        OctTreeNode::<T> {
            depth: parent.depth + 1,
            bounds,
            data_capacity: parent.data_capacity,
            data: vec![],
            children: vec![],
        }
    }

    pub fn insert(&mut self, data: T, aabb: &AABB) -> bool {
        if !self.bounds.intersects(aabb) {
            return false;
        }

        if self.children.is_empty() && self.data.len() < self.data_capacity {
            self.data.push(data);
            return true;
        }

        if self.children.is_empty() {
            self.subdivide();
        }

        for child in self.children.as_mut_slice() {
            if child.insert(data, aabb) {
                return true;
            }
        }

        false
    }

    fn subdivide(&mut self) {
        let child_half_extent = (self.bounds.right - self.bounds.left) / 2.0;

        let left_top_near_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.left,
                y: self.bounds.top,
                z: self.bounds.near,
            },
            0.5,
        );

        let right_top_near_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.right,
                y: self.bounds.top,
                z: self.bounds.near,
            },
            0.5,
        );

        let left_bottom_near_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.left,
                y: self.bounds.bottom,
                z: self.bounds.near,
            },
            0.5,
        );

        let right_bottom_near_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.right,
                y: self.bounds.bottom,
                z: self.bounds.near,
            },
            0.5,
        );

        let left_top_far_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.left,
                y: self.bounds.top,
                z: self.bounds.far,
            },
            0.5,
        );

        let right_top_far_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.right,
                y: self.bounds.top,
                z: self.bounds.far,
            },
            0.5,
        );

        let left_bottom_far_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.left,
                y: self.bounds.bottom,
                z: self.bounds.far,
            },
            0.5,
        );

        let right_bottom_far_center = Vec3::interpolate(
            self.bounds.center,
            Vec3 {
                x: self.bounds.right,
                y: self.bounds.bottom,
                z: self.bounds.far,
            },
            0.5,
        );

        self.children = vec![
            // Left top near
            OctTreeNode::new(self, AABB::cube(left_top_near_center, child_half_extent)),
            // Right top near
            OctTreeNode::new(self, AABB::cube(right_top_near_center, child_half_extent)),
            // Left bottom near
            OctTreeNode::new(self, AABB::cube(left_bottom_near_center, child_half_extent)),
            // Right bottom near
            OctTreeNode::new(
                self,
                AABB::cube(right_bottom_near_center, child_half_extent),
            ),
            // Left top far
            OctTreeNode::new(self, AABB::cube(left_top_far_center, child_half_extent)),
            // Right top far
            OctTreeNode::new(self, AABB::cube(right_top_far_center, child_half_extent)),
            // Left bottom far
            OctTreeNode::new(self, AABB::cube(left_bottom_far_center, child_half_extent)),
            // Right bottom far
            OctTreeNode::new(self, AABB::cube(right_bottom_far_center, child_half_extent)),
        ];
    }
}

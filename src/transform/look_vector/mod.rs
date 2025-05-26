use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    serde::PostDeserialize,
    vec::vec3::{self, Vec3},
};

use super::quaternion::Quaternion;

pub mod controller;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LookVector {
    position: Vec3,
    target: Vec3,
    forward: Vec3,
    up: Vec3,
    right: Vec3,
}

impl PostDeserialize for LookVector {
    fn post_deserialize(&mut self) {}
}

impl fmt::Display for LookVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LookVector (position={}, forward={})",
            self.position, self.forward
        )
    }
}

impl LookVector {
    pub fn new(position: Vec3) -> Self {
        let mut result = Self {
            position,
            target: Default::default(),
            forward: vec3::FORWARD,
            up: vec3::UP,
            right: vec3::RIGHT,
        };

        result.recompute_basis();

        result
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn get_target(&self) -> Vec3 {
        self.target
    }

    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;

        self.recompute_basis();
    }

    pub fn get_forward(&self) -> Vec3 {
        self.forward
    }

    pub fn get_up(&self) -> Vec3 {
        self.up
    }

    pub fn get_right(&self) -> Vec3 {
        self.right
    }

    pub fn apply_rotation(&mut self, q: Quaternion) {
        let rotation = *q.mat();

        self.forward *= rotation;
        self.right *= rotation;
        self.up *= rotation;

        let position_to_target = self.target - self.position;
        let position_to_target_rotated = position_to_target * rotation;

        self.target = self.position + position_to_target_rotated;
    }

    fn recompute_basis(&mut self) {
        let position_to_target = self.target - self.position;

        let (forward, right, up) = position_to_target.basis();

        self.forward = forward;
        self.right = right;
        self.up = up;
    }
}

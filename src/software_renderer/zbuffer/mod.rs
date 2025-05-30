use std::fmt::Display;

use crate::buffer::Buffer2D;

pub static MAX_DEPTH: f32 = 1.0;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum DepthTestMethod {
    #[default]
    // Passes if the fragment's depth is less than the stored depth.
    Less = 0,
    // Passes if the fragment's depth is equal to the stored depth.
    Equal = 1,
    // Passes if the fragment's depth is less than or equal to the stored depth.
    LessThanOrEqual = 2,
    // Passes if the fragment's depth is greater than the stored depth.
    Greater = 3,
    // Passes if the fragment's depth is not equal to the stored depth.
    NotEqual = 4,
    // Passes if the fragment's depth is greater than or equal to the stored depth.
    GreaterThanOrEqual = 5,
    // Always passes.
    Always = 6,
    // Never passes.
    Never = 7,
}

impl Display for DepthTestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DepthTestMethod::Less => "Less",
                DepthTestMethod::Equal => "Equal",
                DepthTestMethod::LessThanOrEqual => "LessThanOrEqual",
                DepthTestMethod::Greater => "Greater",
                DepthTestMethod::NotEqual => "NotEqual",
                DepthTestMethod::GreaterThanOrEqual => "GreaterThanOrEqual",
                DepthTestMethod::Always => "Always",
                DepthTestMethod::Never => "Never",
            }
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZBuffer {
    pub buffer: Buffer2D<f32>,
    projection_z_near: f32,
    projection_z_far: f32,
    projection_z_near_reciprocal: f32,
    projection_z_far_reciprocal: f32,
    projection_depth: f32,
    depth_test_method: DepthTestMethod,
}

impl ZBuffer {
    pub fn new(width: u32, height: u32, projection_z_near: f32, projection_z_far: f32) -> Self {
        let buffer = Buffer2D::<f32>::new(width, height, Some(MAX_DEPTH));

        Self {
            buffer,
            projection_z_near,
            projection_z_near_reciprocal: 1.0 / projection_z_near,
            projection_z_far,
            projection_z_far_reciprocal: 1.0 / projection_z_far,
            projection_depth: projection_z_far - projection_z_near,
            depth_test_method: Default::default(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, f32> {
        self.buffer.iter()
    }

    pub fn get_projection_z_near(&self) -> f32 {
        self.projection_z_near
    }

    pub fn set_projection_z_near(&mut self, depth: f32) {
        self.projection_z_near = depth;

        self.projection_z_near_reciprocal = 1.0 / depth;

        self.projection_depth = self.projection_z_far - self.projection_z_near;
    }

    pub fn get_projection_z_far(&self) -> f32 {
        self.projection_z_far
    }

    pub fn set_projection_z_far(&mut self, depth: f32) {
        self.projection_z_far = depth;

        self.projection_z_far_reciprocal = 1.0 / depth;

        self.projection_depth = self.projection_z_far - self.projection_z_near;
    }

    pub fn get_depth_test_method(&self) -> &DepthTestMethod {
        &self.depth_test_method
    }

    pub fn set_depth_test_method(&mut self, method: DepthTestMethod) {
        self.depth_test_method = method;
    }

    pub fn clear(&mut self) {
        self.buffer.clear(Some(MAX_DEPTH));
    }

    pub fn get_normalized(&self, linear_space_z: f32) -> f32 {
        ((linear_space_z - self.projection_z_near) / self.projection_depth).clamp(0.0, 1.0)
    }

    pub fn test(&mut self, x: u32, y: u32, z: f32) -> Option<((u32, u32), f32)> {
        // Non-linear depth test
        // https://youtu.be/3xGKu4T4SCU?si=v7nkYrg2sFYozfZ5&t=139

        // (1/z - 1/n) / (1/f - 1/n)

        let new_z_non_linear = (1.0 / z - self.projection_z_near_reciprocal)
            / (self.projection_z_far_reciprocal - self.projection_z_near_reciprocal);

        // Check if we can return early.

        match self.depth_test_method {
            DepthTestMethod::Always => return Some(((x, y), new_z_non_linear)),
            DepthTestMethod::Never => return None,
            _ => (),
        }

        // Compare to the current recorded depth, using the appropriate operator.

        let current_z_non_linear = *self.buffer.get(x, y);

        let operator = match self.depth_test_method {
            DepthTestMethod::Less => f32::lt,
            DepthTestMethod::Equal => f32::eq,
            DepthTestMethod::LessThanOrEqual => f32::le,
            DepthTestMethod::Greater => f32::gt,
            DepthTestMethod::NotEqual => f32::ne,
            DepthTestMethod::GreaterThanOrEqual => f32::ge,
            _ => panic!(),
        };

        if operator(&new_z_non_linear, &current_z_non_linear) {
            Some(((x, y), new_z_non_linear))
        } else {
            None
        }
    }

    pub fn set(&mut self, x: u32, y: u32, non_linear_z: f32) {
        self.buffer.set(x, y, non_linear_z)
    }
}

pub static DEPTH_TEST_METHODS: [DepthTestMethod; 8] = [
    DepthTestMethod::Less,
    DepthTestMethod::Equal,
    DepthTestMethod::LessThanOrEqual,
    DepthTestMethod::Greater,
    DepthTestMethod::NotEqual,
    DepthTestMethod::GreaterThanOrEqual,
    DepthTestMethod::Always,
    DepthTestMethod::Never,
];

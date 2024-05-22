use serde::{Deserialize, Serialize};

use crate::vec::{vec3::Vec3, vec4::Vec4};

static NEAR_TOP_LEFT_CLIP_SPACE: Vec4 = Vec4 {
    x: -1.0,
    y: 1.0,
    z: 0.0,
    w: 1.0,
};

static NEAR_TOP_RIGHT_CLIP_SPACE: Vec4 = Vec4 {
    x: 1.0,
    ..NEAR_TOP_LEFT_CLIP_SPACE
};

static NEAR_BOTTOM_LEFT_CLIP_SPACE: Vec4 = Vec4 {
    y: -1.0,
    ..NEAR_TOP_LEFT_CLIP_SPACE
};

static NEAR_BOTTOM_RIGHT_CLIP_SPACE: Vec4 = Vec4 {
    x: 1.0,
    ..NEAR_BOTTOM_LEFT_CLIP_SPACE
};

pub static NEAR_PLANE_POINTS_CLIP_SPACE: [Vec4; 4] = [
    NEAR_TOP_LEFT_CLIP_SPACE,
    NEAR_TOP_RIGHT_CLIP_SPACE,
    NEAR_BOTTOM_RIGHT_CLIP_SPACE,
    NEAR_BOTTOM_LEFT_CLIP_SPACE,
];

pub static FAR_PLANE_POINTS_CLIP_SPACE: [Vec4; 4] = [
    Vec4 {
        z: 1.0,
        ..NEAR_TOP_LEFT_CLIP_SPACE
    },
    Vec4 {
        z: 1.0,
        ..NEAR_TOP_RIGHT_CLIP_SPACE
    },
    Vec4 {
        z: 1.0,
        ..NEAR_BOTTOM_RIGHT_CLIP_SPACE
    },
    Vec4 {
        z: 1.0,
        ..NEAR_BOTTOM_LEFT_CLIP_SPACE
    },
];

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Frustum {
    pub forward: Vec3,
    pub near: [Vec3; 4],
    pub far: [Vec3; 4],
}

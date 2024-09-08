use serde::{Deserialize, Serialize};

use crate::{
    physics::collider::plane::Plane,
    vec::{vec3::Vec3, vec4::Vec4},
};

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

#[derive(Default, Debug, Copy, Clone)]
pub enum NdcPlane {
    #[default]
    Near,
    Far,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Frustum {
    pub forward: Vec3,
    pub near: [Vec3; 4],
    pub far: [Vec3; 4],
}

impl Frustum {
    pub fn get_center(&self) -> Vec3 {
        let near_center = {
            let mut center: Vec3 = Default::default();

            center += self.near[0];
            center += self.near[1];
            center += self.near[2];
            center += self.near[3];

            center /= 4.0;

            center
        };

        let far_center = {
            let mut center: Vec3 = Default::default();

            center += self.far[0];
            center += self.far[1];
            center += self.far[2];
            center += self.far[3];

            center /= 4.0;

            center
        };

        (near_center + far_center) / 2.0
    }

    pub fn get_planes(&self) -> [Plane; 6] {
        let near_top_left = self.near[0];
        let near_top_right = self.near[1];
        let near_bottom_right = self.near[2];
        let near_bottom_left = self.near[3];

        let far_top_left = self.far[0];
        let far_top_right = self.far[1];
        let far_bottom_left = self.far[3];

        let near_normal = self.forward;
        let far_normal = -self.forward;

        let left_normal =
            ((far_top_left - near_top_left).cross(near_bottom_left - near_top_left)).as_normal();

        let right_normal = -((far_top_right - near_top_right)
            .cross(near_bottom_right - near_top_right))
        .as_normal();

        let top_normal =
            ((near_top_right - near_top_left).cross(far_top_left - near_top_left)).as_normal();

        let bottom_normal = -((near_bottom_right - near_bottom_left)
            .cross(far_bottom_left - near_bottom_left))
        .as_normal();

        let near = {
            Plane {
                point: self.near[0],
                normal: near_normal,
            }
        };

        let far = {
            Plane {
                point: self.far[0],
                normal: far_normal,
            }
        };

        let left = {
            Plane {
                point: self.near[0],
                normal: left_normal,
            }
        };

        let right = {
            Plane {
                point: self.near[1],
                normal: right_normal,
            }
        };

        let top = {
            Plane {
                point: self.near[0],
                normal: top_normal,
            }
        };

        let bottom = {
            Plane {
                point: self.near[2],
                normal: bottom_normal,
            }
        };

        [near, far, left, right, top, bottom]
    }
}

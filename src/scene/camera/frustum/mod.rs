use serde::{Deserialize, Serialize};

use crate::{
    geometry::primitives::plane::Plane,
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
    planes: [Plane; 6],
}

impl Frustum {
    pub fn new(forward: Vec3, near: [Vec3; 4], far: [Vec3; 4]) -> Self {
        let planes = make_frustum_planes(forward, &near, &far);

        Self {
            forward,
            near,
            far,
            planes,
        }
    }

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

    pub fn get_planes(&self) -> &[Plane; 6] {
        &self.planes
    }
}

fn make_frustum_planes(forward: Vec3, near: &[Vec3; 4], far: &[Vec3; 4]) -> [Plane; 6] {
    let near_top_left = near[0];
    let near_top_right = near[1];
    let near_bottom_right = near[2];
    let near_bottom_left = near[3];

    let far_top_left = far[0];
    let far_top_right = far[1];
    let far_bottom_left = far[3];

    let near_plane_normal = forward;
    let far_plane_normal = -forward;

    let left_plane_normal =
        ((far_top_left - near_top_left).cross(near_bottom_left - near_top_left)).as_normal();

    let right_plane_normal =
        -((far_top_right - near_top_right).cross(near_bottom_right - near_top_right)).as_normal();

    let top_plane_normal =
        ((near_top_right - near_top_left).cross(far_top_left - near_top_left)).as_normal();

    let bottom_plane_normal = -((near_bottom_right - near_bottom_left)
        .cross(far_bottom_left - near_bottom_left))
    .as_normal();

    let near_plane = Plane::new(near[0], near_plane_normal);
    let far_plane = Plane::new(far[0], far_plane_normal);
    let left_plane = Plane::new(near[0], left_plane_normal);
    let right_plane = Plane::new(near[1], right_plane_normal);
    let top_plane = Plane::new(near[0], top_plane_normal);
    let bottom_plane = Plane::new(near[2], bottom_plane_normal);

    [
        near_plane,
        far_plane,
        left_plane,
        right_plane,
        top_plane,
        bottom_plane,
    ]
}

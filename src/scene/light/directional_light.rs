use std::{
    f32::consts::PI,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

use crate::{
    matrix::Mat4,
    scene::camera::{frustum::Frustum, Camera, CameraOrthographicExtent},
    serde::PostDeserialize,
    shader::geometry::sample::GeometrySample,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::{self, Vec4},
    },
};

use super::contribute_pbr;

pub const DIRECTIONAL_LIGHT_SHADOW_MAP_CAMERAS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub intensities: Vec3,
    rotation: Quaternion,
    direction: Vec4,
    pub shadow_map_cameras: Option<Vec<(f32, Camera)>>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        let mut result = Self {
            intensities: Vec3::ones() * 0.15,
            rotation: Default::default(),
            direction: vec4::FORWARD,
            shadow_map_cameras: Default::default(),
        };

        result.set_direction(Quaternion::new(
            (vec3::RIGHT + vec3::FORWARD).as_normal(),
            PI / 8.0,
        ));

        result
    }
}

impl PostDeserialize for DirectionalLight {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl Display for DirectionalLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DirectionalLight(intensities={}, rotation={}, direction={})",
            self.intensities, self.rotation, self.direction
        )
    }
}

impl DirectionalLight {
    pub fn set_direction(&mut self, rotation: Quaternion) {
        let rotation_mat = *rotation.mat();

        self.direction = vec4::FORWARD * rotation_mat;
    }

    pub fn update_shadow_map_cameras(&mut self, view_camera: &Camera) {
        let forward = self.direction.as_normal().to_vec3();
        let right = vec3::UP.cross(forward).as_normal();
        let up = forward.cross(right).as_normal();

        let alpha_step = 1.0 / DIRECTIONAL_LIGHT_SHADOW_MAP_CAMERAS as f32;

        let view_camera_projection_depth =
            view_camera.get_projection_z_far() - view_camera.get_projection_z_near();

        let projection_depth_step =
            view_camera_projection_depth / DIRECTIONAL_LIGHT_SHADOW_MAP_CAMERAS as f32;

        let frustum = view_camera.get_frustum();

        let subfrustum_cameras: Vec<(f32, Camera)> = (0..DIRECTIONAL_LIGHT_SHADOW_MAP_CAMERAS)
            .map(|subfrustum_index| {
                let near_alpha = alpha_step * subfrustum_index as f32;
                let far_alpha = alpha_step * (subfrustum_index + 1) as f32;

                let subfrustum = Frustum {
                    forward: view_camera.look_vector.get_forward(),
                    near: [
                        Vec3::interpolate(frustum.near[0], frustum.far[0], near_alpha),
                        Vec3::interpolate(frustum.near[1], frustum.far[1], near_alpha),
                        Vec3::interpolate(frustum.near[2], frustum.far[2], near_alpha),
                        Vec3::interpolate(frustum.near[3], frustum.far[3], near_alpha),
                    ],
                    far: [
                        Vec3::interpolate(frustum.near[0], frustum.far[0], far_alpha),
                        Vec3::interpolate(frustum.near[1], frustum.far[1], far_alpha),
                        Vec3::interpolate(frustum.near[2], frustum.far[2], far_alpha),
                        Vec3::interpolate(frustum.near[3], frustum.far[3], far_alpha),
                    ],
                };

                let subfrustum_far_z = projection_depth_step * (subfrustum_index + 1) as f32;

                let subfrustum_center = subfrustum.get_center();

                let mut min_z = f32::MAX;
                let mut max_z = f32::MIN;

                let light_extent = {
                    let mut min_x = f32::MAX;
                    let mut max_x = f32::MIN;
                    let mut min_y = f32::MAX;
                    let mut max_y = f32::MIN;

                    let light_view_inverse_transform =
                        Mat4::look_at(subfrustum_center, forward, right, up);

                    for coord in subfrustum.near.iter().chain(subfrustum.far.iter()) {
                        let view_space_coord =
                            Vec4::new(*coord, 1.0) * light_view_inverse_transform;

                        min_x = min_x.min(view_space_coord.x);
                        max_x = max_x.max(view_space_coord.x);

                        min_y = min_y.min(view_space_coord.y);
                        max_y = max_y.max(view_space_coord.y);

                        min_z = min_z.min(view_space_coord.z);
                        max_z = max_z.max(view_space_coord.z);
                    }

                    CameraOrthographicExtent {
                        left: min_x,
                        right: max_x,
                        top: max_y,
                        bottom: min_y,
                    }
                };

                let depth_range = max_z - min_z;

                let camera_position = subfrustum_center - forward * depth_range;

                let mut camera = Camera::from_orthographic(
                    camera_position,
                    camera_position + self.direction.to_vec3(),
                    light_extent,
                );

                camera.set_projection_z_near(0.2);
                camera.set_projection_z_far(depth_range * 2.0);

                camera.recompute_world_space_frustum();

                (subfrustum_far_z, camera)
            })
            .collect();

        self.shadow_map_cameras = Some(subfrustum_cameras);
    }

    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let normal = &tangent_space_info.normal;

        let direction_to_light = (self.direction * -1.0 * tangent_space_info.tbn_inverse)
            .to_vec3()
            .as_normal();

        self.intensities * 0.0_f32.max((*normal * -1.0).dot(direction_to_light))
    }

    pub fn contribute_pbr(&self, sample: &GeometrySample, f0: &Vec3) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let direction_to_light = (self.direction * -1.0 * tangent_space_info.tbn_inverse)
            .to_vec3()
            .as_normal();

        contribute_pbr(sample, &self.intensities, &direction_to_light, f0)
    }
}

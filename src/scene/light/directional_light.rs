use std::{
    f32::consts::PI,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

use crate::{
    color,
    matrix::Mat4,
    scene::camera::{frustum::Frustum, Camera, CameraOrthographicExtent},
    serde::PostDeserialize,
    shader::{context::ShaderContext, geometry::sample::GeometrySample},
    texture::{map::TextureMap, sample::sample_nearest_f32},
    transform::quaternion::Quaternion,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::{self, Vec4},
    },
};

use super::contribute_pbr;

pub const SHADOW_MAP_CAMERA_NEAR: f32 = 0.05;

pub const SHADOW_MAP_CAMERA_COUNT: usize = 3;

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
            shadow_map_cameras: None,
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
    pub fn get_direction(&mut self) -> &Vec4 {
        &self.direction
    }

    pub fn set_direction(&mut self, rotation: Quaternion) {
        let rotation_mat = *rotation.mat();

        self.direction = vec4::FORWARD * rotation_mat;
    }

    pub fn update_shadow_map_cameras(&mut self, view_camera: &Camera) {
        let forward = self.direction.as_normal().to_vec3();
        let right = vec3::UP.cross(forward).as_normal();
        let up = forward.cross(right).as_normal();

        let alpha_step = 1.0 / SHADOW_MAP_CAMERA_COUNT as f32;

        let view_camera_projection_depth =
            view_camera.get_projection_z_far() - view_camera.get_projection_z_near();

        let projection_depth_step = view_camera_projection_depth / SHADOW_MAP_CAMERA_COUNT as f32;

        let frustum = view_camera.get_frustum();

        let subfrustum_cameras: Vec<(f32, Camera)> = (0..SHADOW_MAP_CAMERA_COUNT)
            .map(|subfrustum_index| {
                let near_alpha = alpha_step * subfrustum_index as f32;
                let far_alpha = alpha_step * (subfrustum_index + 1) as f32;

                let subfrustum = Frustum {
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
                    forward,
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

                camera.set_projection_z_near(SHADOW_MAP_CAMERA_NEAR);
                camera.set_projection_z_far(depth_range * 2.0);

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

    pub fn contribute_pbr(
        &self,
        sample: &GeometrySample,
        f0: &Vec3,
        context: &ShaderContext,
    ) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let direction_to_light = (self.direction * -1.0 * tangent_space_info.tbn_inverse)
            .to_vec3()
            .as_normal();

        // Compute an enshadowing term for this fragment/sample.

        let (shadow_map_index, in_shadow) = self.get_shadowing(sample, context);

        let _shadow_map_index_color = match shadow_map_index {
            0 => color::RED.to_vec3() / 255.0,
            1 => color::GREEN.to_vec3() / 255.0,
            2 => color::BLUE.to_vec3() / 255.0,
            _ => panic!(),
        };

        let intensity = self.intensities;

        let contribution = contribute_pbr(sample, &intensity, &direction_to_light, f0);

        contribution * (1.0 - in_shadow)
    }

    fn get_shadowing_for_map(
        &self,
        sample: &GeometrySample,
        map: &TextureMap<f32>,
        _far_z: f32,
        transform: &Mat4,
    ) -> f32 {
        let sample_position_light_view_projection_space =
            Vec4::new(sample.world_pos, 1.0) * *transform;

        let sample_position_light_ndc_space = sample_position_light_view_projection_space
            / sample_position_light_view_projection_space.w;

        let current_depth = sample_position_light_ndc_space.z;

        let uv = Vec2 {
            x: 0.5 + sample_position_light_ndc_space.x / 2.0,
            y: 0.5 + sample_position_light_ndc_space.y / 2.0,
            z: 0.0,
        };

        let texel_size = 1.0 / map.width as f32;

        let mut shadow = 0.0;

        for y in -1..1 {
            for x in -1..1 {
                if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
                    continue;
                }

                let depth_sample = sample_nearest_f32(
                    uv + Vec2 {
                        x: x as f32,
                        y: y as f32,
                        z: 0.0,
                    } * texel_size,
                    map,
                );

                let closest_depth = depth_sample * 100.0;

                if closest_depth == 0.0 {
                    continue;
                }

                let bias = -0.01;

                let is_in_shadow = current_depth + bias > closest_depth;

                if is_in_shadow {
                    shadow += 1.0;
                }
            }
        }

        shadow / 9.0
    }

    fn get_shadowing(&self, sample: &GeometrySample, context: &ShaderContext) -> (usize, f32) {
        match (
            &context.directional_light_shadow_maps,
            &context.directional_light_view_projections,
        ) {
            (Some(maps), Some(transforms)) => {
                let fragment_position_view_space =
                    Vec4::new(sample.world_pos, 1.0) * context.view_inverse_transform;

                let index = {
                    let mut index = SHADOW_MAP_CAMERA_COUNT - 1;

                    for (i, transform) in transforms.iter().enumerate() {
                        let (far_z, _transform) = transform;

                        if fragment_position_view_space.z.abs() < *far_z {
                            index = i;

                            break;
                        }
                    }

                    index
                };

                let shadowing = self.get_shadowing_for_map(
                    sample,
                    &maps[index],
                    transforms[index].0,
                    &transforms[index].1,
                );

                (index, shadowing)
            }
            _ => (0, 0.0),
        }
    }
}

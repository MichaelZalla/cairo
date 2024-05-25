use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    color,
    matrix::Mat4,
    physics::pbr::{self, brdf::cook_torrance_direct},
    resource::handle::Handle,
    serde::PostDeserialize,
    shader::geometry::sample::GeometrySample,
    texture::cubemap::CubeMap,
    transform::look_vector::LookVector,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::camera::{frustum::Frustum, Camera, CameraOrthographicExtent};

fn contribute_pbr(
    sample: &GeometrySample,
    light_intensities: &Vec3,
    direction_to_light: &Vec3,
    f0: &Vec3,
) -> Vec3 {
    let tangent_space_info = sample.tangent_space_info;

    let normal = &tangent_space_info.normal;

    let direction_to_view_position =
        (tangent_space_info.view_position - tangent_space_info.fragment_position).as_normal();

    let likeness_to_light_direction = normal.dot(*direction_to_light).max(0.0);

    if likeness_to_light_direction > 0.0 {
        let radiance = *light_intensities;

        let halfway = (direction_to_view_position + *direction_to_light).as_normal();

        let halfway_likeness_to_view = halfway.dot(direction_to_view_position);

        let fresnel = pbr::brdf::fresnel_schlick_direct(halfway_likeness_to_view, f0);

        // Rendering equation

        // Ratio of reflected light energy.
        let k_s = fresnel;

        // Ratio of refracted light energy.
        let k_d = (vec3::ONES - k_s) * (1.0 - sample.metallic);

        // BRDF

        let diffuse = k_d * (sample.albedo / PI);

        let likeness_to_view_direction = normal.dot(direction_to_view_position).max(0.0);

        let specular = cook_torrance_direct(
            sample,
            &halfway,
            &direction_to_view_position,
            likeness_to_view_direction,
            direction_to_light,
            likeness_to_light_direction,
            &fresnel,
        );

        (diffuse + specular) * radiance * likeness_to_light_direction
    } else {
        Default::default()
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AmbientLight {
    pub intensities: Vec3,
}

impl PostDeserialize for AmbientLight {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl AmbientLight {
    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        self.intensities * sample.ambient_factor
    }

    pub fn contribute_pbr(self, sample: &GeometrySample) -> Vec3 {
        self.intensities * sample.albedo * sample.ambient_factor
    }
}

pub const DIRECTIONAL_LIGHT_SHADOW_MAP_CAMERAS: usize = 3;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub intensities: Vec3,
    pub direction: Vec4,
    pub shadow_map_cameras: Option<Vec<(f32, Camera)>>,
}

impl PostDeserialize for DirectionalLight {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl DirectionalLight {
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

pub static POINT_LIGHT_SHADOW_MAP_SIZE: u32 = 512;
pub static POINT_LIGHT_SHADOW_CAMERA_NEAR: f32 = 0.3;
pub static POINT_LIGHT_SHADOW_CAMERA_FAR: f32 = 100.0;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub intensities: Vec3,
    pub position: Vec3,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    #[serde(skip)]
    pub shadow_map: Option<Handle>,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for PointLight {
    fn post_deserialize(&mut self) {
        self.influence_distance = get_approximate_influence_distance(
            self.quadratic_attenuation,
            self.linear_attenuation,
            self.constant_attenuation,
        );
    }
}

impl PointLight {
    pub fn new() -> Self {
        let mut light = PointLight {
            intensities: color::WHITE.to_vec3() / 255.0,
            position: Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            constant_attenuation: 1.0,
            linear_attenuation: 0.35,
            quadratic_attenuation: 0.44,
            shadow_map: None,
            influence_distance: 0.0,
        };

        light.post_deserialize();

        light
    }

    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        let mut point_contribution: Vec3 = Vec3::new();
        let mut specular_contribution: Vec3 = Vec3::new();

        let tangent_space_info = sample.tangent_space_info;

        let normal = &tangent_space_info.normal;

        let point_light_position_tangent_space =
            (Vec4::new(self.position, 1.0) * tangent_space_info.tbn_inverse).to_vec3();

        let fragment_to_point_light_tangent_space =
            point_light_position_tangent_space - tangent_space_info.fragment_position;

        let distance_to_point_light_tangent_space = fragment_to_point_light_tangent_space.mag();

        let direction_to_point_light_tangent_space =
            fragment_to_point_light_tangent_space / distance_to_point_light_tangent_space;

        let likeness = 0.0_f32.max(normal.dot(direction_to_point_light_tangent_space));

        if likeness > 0.0 {
            let attenuation = self.get_attentuation(distance_to_point_light_tangent_space);

            point_contribution = self.intensities * attenuation * 0.0_f32.max(likeness);

            let reflected_ray = {
                // Calculate specular light intensity
                let incoming_ray = fragment_to_point_light_tangent_space * -1.0;

                // Project the incoming ray forward through the fragment/surface
                let absorbed_ray = tangent_space_info.fragment_position + incoming_ray;

                // Project the incoming light ray onto the surface normal (i.e.,
                // scaling the normal up or down)
                let w = *normal * incoming_ray.dot(*normal);

                // Combine the absorbed ray with the scaled normal to find the
                // reflected ray vector.
                let u = w * 2.0;

                u - absorbed_ray
            };

            // Get the reflected ray's direction from the surface
            let reflected_ray_normal = reflected_ray.as_normal();

            // Compute the similarity between the reflected ray's direction and
            // the direction from our fragment to the viewer.
            let fragment_to_view_tangent_space =
                tangent_space_info.view_position - tangent_space_info.fragment_position;

            let view_direction_normal = fragment_to_view_tangent_space.as_normal();

            let cosine_theta = 1.0_f32.min(reflected_ray_normal.dot(view_direction_normal * -1.0));

            let similarity = 0.0_f32.max(cosine_theta);

            specular_contribution = point_contribution
                * sample.specular_color
                * similarity.powi(sample.specular_exponent as i32);
        }

        point_contribution + specular_contribution
    }

    pub fn contribute_pbr(
        &self,
        sample: &GeometrySample,
        f0: &Vec3,
        shadow_map: Option<&CubeMap<f32>>,
    ) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let point_light_position =
            (Vec4::new(self.position, 1.0) * tangent_space_info.tbn_inverse).to_vec3();

        let fragment_to_point_light = point_light_position - tangent_space_info.fragment_position;
        let distance_to_point_light = fragment_to_point_light.mag();
        let direction_to_point_light = fragment_to_point_light / distance_to_point_light;

        // Compute an enshadowing term for this fragment/sample.

        let in_shadow = if let Some(map) = shadow_map {
            self.get_shadowing(sample, map)
        } else {
            0.0
        };

        let contribution = contribute_pbr(sample, &self.intensities, &direction_to_point_light, f0)
            * self.get_attentuation(distance_to_point_light);

        contribution * (1.0 - in_shadow)
    }

    fn get_shadowing(&self, sample: &GeometrySample, shadow_map: &CubeMap<f32>) -> f32 {
        let light_to_fragment = sample.world_pos - self.position;
        let light_to_fragment_direction = light_to_fragment.as_normal();

        let current_depth = light_to_fragment.mag();

        let closest_depth = shadow_map.sample_nearest(&Vec4::new(light_to_fragment_direction, 1.0))
            * POINT_LIGHT_SHADOW_CAMERA_FAR;

        if closest_depth == 0.0 {
            return 0.0;
        }

        let likeness = sample
            .world_normal
            .dot((self.position - sample.world_pos).as_normal());

        let bias = 0.005_f32.max(0.05 * (1.0 - likeness));

        let is_in_shadow = (current_depth + bias) > closest_depth;

        if is_in_shadow {
            1.0
        } else {
            0.0
        }
    }

    #[inline]
    fn get_attentuation(&self, distance: f32) -> f32 {
        1.0 / (self.quadratic_attenuation * distance.powi(2)
            + self.linear_attenuation * distance
            + self.constant_attenuation)
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SpotLight {
    pub intensities: Vec3,
    pub look_vector: LookVector,
    pub inner_cutoff_angle: f32,
    #[serde(skip)]
    pub inner_cutoff_angle_cos: f32,
    pub outer_cutoff_angle: f32,
    #[serde(skip)]
    pub outer_cutoff_angle_cos: f32,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for SpotLight {
    fn post_deserialize(&mut self) {
        self.influence_distance = get_approximate_influence_distance(
            self.quadratic_attenuation,
            self.linear_attenuation,
            self.constant_attenuation,
        );
    }
}

impl SpotLight {
    pub fn new() -> Self {
        let default_light_position = Vec3 {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        };

        let mut light = SpotLight {
            intensities: Vec3 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            },
            look_vector: LookVector::new(
                default_light_position,
                default_light_position + vec3::UP * -1.0,
            ),
            inner_cutoff_angle: (PI / 12.0),
            outer_cutoff_angle: (PI / 8.0),
            inner_cutoff_angle_cos: (PI / 12.0).cos(),
            outer_cutoff_angle_cos: (PI / 8.0).cos(),
            constant_attenuation: 1.0,
            linear_attenuation: 0.09,
            quadratic_attenuation: 0.032,
            influence_distance: 0.0,
        };

        light.post_deserialize();

        light
    }

    pub fn contribute(self, world_pos: Vec3) -> Vec3 {
        let mut spot_light_contribution: Vec3 = Vec3::new();

        let vertex_to_spot_light = self.look_vector.get_position() - world_pos;

        let distance_to_spot_light = vertex_to_spot_light.mag();

        let direction_to_spot_light = vertex_to_spot_light / distance_to_spot_light;

        let theta_angle =
            0.0_f32.max((self.look_vector.get_forward()).dot(direction_to_spot_light * -1.0));

        let epsilon = self.inner_cutoff_angle_cos - self.outer_cutoff_angle_cos;

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle_cos {
            spot_light_contribution = self.intensities * spot_attenuation;
        }

        spot_light_contribution
    }

    pub fn contribute_pbr(&self, sample: &GeometrySample, f0: &Vec3) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let spot_light_position_tangent_space = (Vec4::new(self.look_vector.get_position(), 1.0)
            * tangent_space_info.tbn_inverse)
            .to_vec3();

        let direction_to_light =
            (spot_light_position_tangent_space - tangent_space_info.fragment_position).as_normal();

        let theta_angle =
            0.0_f32.max((self.look_vector.get_forward()).dot(direction_to_light * -1.0));

        let epsilon = self.inner_cutoff_angle_cos - self.outer_cutoff_angle_cos;

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle_cos {
            return contribute_pbr(sample, &self.intensities, &direction_to_light, f0)
                * spot_attenuation;
        }

        Default::default()
    }
}

fn get_approximate_influence_distance(
    quadratic_attenuation: f32,
    linear_attenuation: f32,
    constant_attenuation: f32,
) -> f32 {
    // y = 1.0 / (0.35x + 0.44x^2 + 1), from -20.0 to 20.0

    let mut distance: f32 = 0.01;

    let mut attenuation = 1.0
        / (quadratic_attenuation * distance * distance
            + linear_attenuation * distance
            + constant_attenuation);

    while attenuation > 0.1 {
        // while attenuation > 0.0001 {
        distance += 0.01;
        attenuation = 1.0
            / (quadratic_attenuation * distance * distance
                + linear_attenuation * distance
                + constant_attenuation);
    }

    distance -= 0.01;

    distance
}

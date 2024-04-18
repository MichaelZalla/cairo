use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use crate::color;
use crate::shader::geometry::sample::GeometrySample;
use crate::transform::look_vector::LookVector;
use crate::vec::vec3::{self, Vec3};
use crate::vec::vec4::Vec4;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AmbientLight {
    pub intensities: Vec3,
}

impl AmbientLight {
    pub fn contribute(self, ambient_intensity_factor: f32) -> Vec3 {
        return self.intensities * ambient_intensity_factor;
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub intensities: Vec3,
    pub direction: Vec4,
}

impl DirectionalLight {
    pub fn contribute(self, surface_normal: Vec3) -> Vec3 {
        self.intensities
            * (0.0 as f32).max((surface_normal * -1.0).dot(Vec3 {
                x: self.direction.x,
                y: self.direction.y,
                z: self.direction.z,
            }))
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub intensities: Vec3,
    pub specular_intensity: f32,
    pub position: Vec3,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PointLight {
    pub fn new() -> Self {
        let mut light = PointLight {
            intensities: color::WHITE.to_vec3() / 255.0,
            specular_intensity: 0.5,
            position: Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            constant_attenuation: 1.0,
            linear_attenuation: 0.35,
            quadratic_attenuation: 0.44,
            influence_distance: 0.0,
        };

        light.influence_distance = get_approximate_influence_distance(
            light.quadratic_attenuation,
            light.linear_attenuation,
            light.constant_attenuation,
        );

        light
    }

    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        let mut point_contribution: Vec3 = Vec3::new();
        let mut specular_contribution: Vec3 = Vec3::new();

        let tangent_space_info = sample.tangent_space_info;

        let fragment_to_point_light_tangent_space =
            tangent_space_info.point_light_position - tangent_space_info.fragment_position;

        let distance_to_point_light_tangent_space = fragment_to_point_light_tangent_space.mag();

        let direction_to_point_light_tangent_space =
            fragment_to_point_light_tangent_space / distance_to_point_light_tangent_space;

        let likeness = (0.0 as f32).max(
            sample
                .tangent_space_info
                .normal
                .dot(direction_to_point_light_tangent_space),
        );

        if likeness > 0.0 {
            let attentuation = 1.0
                / (self.quadratic_attenuation * distance_to_point_light_tangent_space.powi(2)
                    + self.linear_attenuation * distance_to_point_light_tangent_space
                    + self.constant_attenuation);

            point_contribution = self.intensities * attentuation * (0.0 as f32).max(likeness);

            // Calculate specular light intensity

            let incoming_ray = fragment_to_point_light_tangent_space * -1.0;

            // Project the incoming ray forward through the fragment/surface
            let absorbed_ray = tangent_space_info.fragment_position + incoming_ray;

            // Project the incoming light ray onto the surface normal (i.e.,
            // scaling the normal up or down)
            let w = tangent_space_info.normal * incoming_ray.dot(tangent_space_info.normal);

            // Combine the absorbed ray with the scaled normal to find the
            // reflected ray vector.
            let u = w * 2.0;
            let reflected_ray = u - absorbed_ray;

            // Get the reflected ray's direction from the surface
            let reflected_ray_normal = reflected_ray.as_normal();

            // Compute the similarity between the reflected ray's direction and
            // the direction from our fragment to the viewer.
            let fragment_to_view_tangent_space =
                tangent_space_info.view_position - tangent_space_info.fragment_position;

            let view_direction_normal = fragment_to_view_tangent_space.as_normal();

            let cosine_theta =
                (1.0 as f32).min(reflected_ray_normal.dot(view_direction_normal * -1.0));

            let similarity = (0.0 as f32).max(cosine_theta);

            specular_contribution = point_contribution
                * sample.specular_intensity
                * similarity.powi(sample.specular_exponent);
        }

        return point_contribution + specular_contribution;
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

        light.influence_distance = get_approximate_influence_distance(
            light.quadratic_attenuation,
            light.linear_attenuation,
            light.constant_attenuation,
        );

        light
    }

    pub fn contribute(self, world_pos: Vec3) -> Vec3 {
        let mut spot_light_contribution: Vec3 = Vec3::new();

        let vertex_to_spot_light = self.look_vector.get_position() - world_pos;

        let distance_to_spot_light = vertex_to_spot_light.mag();

        let direction_to_spot_light = vertex_to_spot_light / distance_to_spot_light;

        let theta_angle =
            (0.0 as f32).max((self.look_vector.get_forward()).dot(direction_to_spot_light * -1.0));

        let epsilon = self.inner_cutoff_angle_cos - self.outer_cutoff_angle_cos;

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle_cos {
            spot_light_contribution = self.intensities * spot_attenuation;
        }

        spot_light_contribution
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

use std::f32::consts::PI;

use crate::color;
use crate::vec::vec3::Vec3;
use crate::vec::vec4::Vec4;

#[derive(Debug, Copy, Clone)]
pub struct AmbientLight {
    pub intensities: Vec3,
}

impl AmbientLight {
    pub fn contribute(self, ambient_intensity_factor: f32) -> Vec3 {
        return self.intensities * ambient_intensity_factor;
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub intensities: Vec3,
    pub specular_intensity: f32,
    pub position: Vec3,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
}

impl PointLight {
    pub fn new() -> Self {
        PointLight {
            intensities: color::WHITE.to_vec3() / 255.0,
            specular_intensity: 0.5,
            position: Default::default(),
            constant_attenuation: 1.0,
            linear_attenuation: 0.35,
            quadratic_attenuation: 0.44,
        }
    }
}

impl PointLight {
    pub fn contribute(
        self,
        world_pos: Vec3,
        surface_normal: Vec3,
        view_position: Vec4,
        specular_intensity: f32,
        specular_exponent: i32,
    ) -> Vec3 {
        let mut point_contribution: Vec3 = Vec3::new();
        let mut specular_contribution: Vec3 = Vec3::new();

        let vertex_to_point_light = self.position - world_pos;

        let distance_to_point_light = vertex_to_point_light.mag();

        let direction_to_point_light = vertex_to_point_light / distance_to_point_light;

        let likeness = (0.0 as f32).max(surface_normal.dot(direction_to_point_light));

        if likeness > 0.0 {
            let attentuation = 1.0
                / (self.quadratic_attenuation * distance_to_point_light.powi(2)
                    + self.linear_attenuation * distance_to_point_light
                    + self.constant_attenuation);

            point_contribution = self.intensities * attentuation * (0.0 as f32).max(likeness);

            // Calculate specular light intensity

            let incoming_ray = vertex_to_point_light * -1.0;

            // Project the incoming ray forward through the fragment/surface
            let absorbed_ray = world_pos + incoming_ray;

            // Project the incoming light ray onto the surface normal (i.e.,
            // scaling the normal up or down)
            let w = surface_normal * incoming_ray.dot(surface_normal);

            // Combine the absorbed ray with the scaled normal to find the
            // reflected ray vector.
            let u = w * 2.0;
            let reflected_ray = u - absorbed_ray;

            // Get the reflected ray's direction from the surface
            let reflected_ray_normal = reflected_ray.as_normal();

            // Compute the similarity between the reflected ray's direction and
            // the direction from our fragment to the viewer.
            let view_direction_normal = (Vec3 {
                x: view_position.x,
                y: view_position.y,
                z: view_position.z,
            } - world_pos)
                .as_normal();

            let cosine_theta =
                (1.0 as f32).min(reflected_ray_normal.dot(view_direction_normal * -1.0));

            let similarity = (0.0 as f32).max(cosine_theta);

            specular_contribution =
                point_contribution * specular_intensity * similarity.powi(specular_exponent);
        }

        return point_contribution + specular_contribution;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SpotLight {
    pub intensities: Vec3,
    pub position: Vec3,
    pub direction: Vec3,
    pub inner_cutoff_angle: f32,
    pub outer_cutoff_angle: f32,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
}

impl SpotLight {
    pub fn new() -> Self {
        SpotLight {
            intensities: Vec3 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            },
            position: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            direction: Vec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            inner_cutoff_angle: (PI / 45.0).cos(),
            outer_cutoff_angle: (PI / 25.0).cos(),
            constant_attenuation: 1.0,
            linear_attenuation: 0.35,
            quadratic_attenuation: 0.44,
        }
    }

    pub fn contribute(self, world_pos: Vec3) -> Vec3 {
        let mut spot_light_contribution: Vec3 = Vec3::new();

        let vertex_to_spot_light = self.position - world_pos;

        let distance_to_spot_light = vertex_to_spot_light.mag();

        let direction_to_spot_light = vertex_to_spot_light / distance_to_spot_light;

        let theta_angle = (0.0 as f32).max((self.direction).dot(direction_to_spot_light * -1.0));

        let epsilon = self.inner_cutoff_angle - self.outer_cutoff_angle;

        let spot_attenuation = ((theta_angle - self.outer_cutoff_angle) / epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle {
            spot_light_contribution = self.intensities * spot_attenuation;
        }

        spot_light_contribution
    }
}

use std::{
    f32::consts::PI,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

use crate::{
    serde::PostDeserialize,
    shader::geometry::sample::GeometrySample,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::{self, Vec4},
    },
};

use super::contribute_pbr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub intensities: Vec3,
    rotation: Quaternion,
    direction: Vec4,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        let mut result = Self {
            intensities: Vec3::ones() * 0.15,
            rotation: Default::default(),
            direction: vec4::FORWARD,
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

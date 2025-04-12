use std::{
    f32::consts::PI,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

use crate::{
    serde::PostDeserialize,
    shader::geometry::sample::GeometrySample,
    transform::look_vector::LookVector,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::{
    attenuation::{LightAttenuation, LIGHT_ATTENUATION_RANGE_50_UNITS},
    contribute_pbr_world_space,
};

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
    #[serde(skip)]
    epsilon: f32,
    attenuation: LightAttenuation,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for SpotLight {
    fn post_deserialize(&mut self) {
        self.recompute_influence_distance();
    }
}

impl Display for SpotLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpotLight (intensities={}, look_vector={})",
            self.intensities, self.look_vector
        )
    }
}

impl SpotLight {
    pub fn new() -> Self {
        let mut light = SpotLight {
            intensities: vec3::ONES,
            look_vector: LookVector::new(Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            }),
            inner_cutoff_angle: (PI / 12.0),
            outer_cutoff_angle: (PI / 8.0),
            inner_cutoff_angle_cos: (PI / 12.0).cos(),
            outer_cutoff_angle_cos: (PI / 8.0).cos(),
            attenuation: LIGHT_ATTENUATION_RANGE_50_UNITS,
            ..Default::default()
        };

        light.look_vector.set_target(-vec3::UP);

        light.epsilon = light.inner_cutoff_angle_cos - light.outer_cutoff_angle_cos;

        light.post_deserialize();

        light
    }

    pub fn get_attenuation(&self) -> &LightAttenuation {
        &self.attenuation
    }

    pub fn set_attenuation(&mut self, attenuation: LightAttenuation) {
        self.attenuation = attenuation;

        self.recompute_influence_distance();
    }

    fn recompute_influence_distance(&mut self) {
        self.influence_distance = self.attenuation.get_approximate_influence_distance();
    }

    pub fn contribute(self, world_pos: Vec3) -> Vec3 {
        let fragment_to_light = self.look_vector.get_position() - world_pos;

        let direction_to_light = fragment_to_light.as_normal();

        let theta_angle =
            0.0_f32.max((self.look_vector.get_forward()).dot(direction_to_light * -1.0));

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / self.epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle_cos {
            self.intensities * spot_attenuation
        } else {
            Default::default()
        }
    }

    pub fn contribute_pbr(&self, sample: &GeometrySample, f0: &Vec3, view_position: &Vec4) -> Vec3 {
        let fragment_to_light = self.look_vector.get_position() - sample.position_world_space;

        let direction_to_light_world_space = fragment_to_light.as_normal();

        let theta_angle = 0.0_f32
            .max((self.look_vector.get_forward()).dot(direction_to_light_world_space * -1.0));

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / self.epsilon).clamp(0.0, 1.0);

        let light_intensities = &self.intensities;

        if theta_angle > self.outer_cutoff_angle_cos {
            contribute_pbr_world_space(
                sample,
                light_intensities,
                &direction_to_light_world_space,
                f0,
                view_position,
            ) * spot_attenuation
        } else {
            Default::default()
        }
    }
}

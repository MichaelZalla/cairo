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

use super::{attenuation::LightAttenuation, contribute_pbr};

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
    pub attenuation: LightAttenuation,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for SpotLight {
    fn post_deserialize(&mut self) {
        self.influence_distance = self.attenuation.get_approximate_influence_distance();
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
            intensities: vec3::ONES * 0.5,
            look_vector: LookVector::new(
                Default::default(),
                Vec3 {
                    x: 0.001,
                    y: -1.0,
                    z: 0.001,
                },
            ),
            inner_cutoff_angle: (PI / 12.0),
            outer_cutoff_angle: (PI / 8.0),
            inner_cutoff_angle_cos: (PI / 12.0).cos(),
            outer_cutoff_angle_cos: (PI / 8.0).cos(),
            attenuation: LightAttenuation::new(1.0, 0.09, 0.032),
            ..Default::default()
        };

        light.epsilon = light.inner_cutoff_angle_cos - light.outer_cutoff_angle_cos;

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

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / self.epsilon).clamp(0.0, 1.0);

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

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / self.epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle_cos {
            contribute_pbr(sample, &self.intensities, &direction_to_light, f0) * spot_attenuation
        } else {
            Default::default()
        }
    }
}

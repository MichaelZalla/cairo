use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LightAttenuation {
    constant: f32,
    linear: f32,
    quadratic: f32,
}

impl Default for LightAttenuation {
    fn default() -> Self {
        Self::new(1.0, 0.09, 0.032)
    }
}

impl LightAttenuation {
    pub const fn new(constant: f32, linear: f32, quadratic: f32) -> Self {
        Self {
            constant,
            linear,
            quadratic,
        }
    }

    pub fn attenuate_for_distance(&self, distance: f32) -> f32 {
        1.0 / (self.quadratic * distance * distance + self.linear * distance + self.constant)
    }

    pub fn get_approximate_influence_distance(&self) -> f32 {
        let mut distance: f32 = 0.01;

        let mut attenuation = self.attenuate_for_distance(distance);

        while attenuation > 0.1 {
            distance += 0.01;

            attenuation = self.attenuate_for_distance(distance);
        }

        distance -= 0.01;

        distance
    }
}

// See: https://wiki.ogre3d.org/-Point+Light+Attenuation

pub static LIGHT_ATTENUATION_RANGE_7_UNITS: LightAttenuation = LightAttenuation::new(1.0, 0.7, 1.8);

pub static LIGHT_ATTENUATION_RANGE_13_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.35, 0.44);

pub static LIGHT_ATTENUATION_RANGE_20_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.22, 0.20);

pub static LIGHT_ATTENUATION_RANGE_32_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.14, 0.07);

pub static LIGHT_ATTENUATION_RANGE_50_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.09, 0.032);

pub static LIGHT_ATTENUATION_RANGE_65_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.07, 0.017);

pub static LIGHT_ATTENUATION_RANGE_100_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.045, 0.0075);

pub static LIGHT_ATTENUATION_RANGE_160_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.027, 0.0028);

pub static LIGHT_ATTENUATION_RANGE_200_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.022, 0.0019);

pub static LIGHT_ATTENUATION_RANGE_325_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.014, 0.0007);

pub static LIGHT_ATTENUATION_RANGE_600_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.007, 0.0002);

pub static LIGHT_ATTENUATION_RANGE_3250_UNITS: LightAttenuation =
    LightAttenuation::new(1.0, 0.0014, 0.000007);

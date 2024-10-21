use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::{serde::PostDeserialize, shader::geometry::sample::GeometrySample, vec::vec3::Vec3};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AmbientLight {
    pub intensities: Vec3,
}

impl PostDeserialize for AmbientLight {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl Display for AmbientLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AmbientLight (intensities={})", self.intensities)
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

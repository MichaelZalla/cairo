use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{resource::handle::Handle, serde::PostDeserialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Skybox {
    pub is_hdr: bool,
    pub radiance: Option<Handle>,
    pub irradiance: Option<Handle>,
    pub specular_prefiltered_environment: Option<Handle>,
}

impl PostDeserialize for Skybox {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl fmt::Display for Skybox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skybox (is_hdr={})", self.is_hdr)
    }
}

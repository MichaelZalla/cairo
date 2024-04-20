use serde::{Deserialize, Serialize};

use crate::{resource::handle::Handle, serde::PostDeserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub mesh: Handle,
    pub material: Option<String>,
}

impl PostDeserialize for Entity {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl Entity {
    pub fn new(mesh: Handle, material: Option<String>) -> Self {
        Self { mesh, material }
    }
}

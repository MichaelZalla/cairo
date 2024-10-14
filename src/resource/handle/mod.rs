use std::str::FromStr;

use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Handle {
    pub index: usize,
    pub uuid: Uuid,
}

impl Handle {
    pub fn from_uuid(index: usize, uuid: &Uuid) -> Self {
        Self { index, uuid: *uuid }
    }

    pub fn from_uuid_str(index: usize, uuid: &str) -> Result<Self, String> {
        match Uuid::from_str(uuid) {
            Ok(uuid) => Ok(Self { index, uuid }),
            Err(_) => Err(format!("Failed to parse a UUID from string '{}'.", uuid)),
        }
    }
}

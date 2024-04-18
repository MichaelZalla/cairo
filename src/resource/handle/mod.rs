use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Handle {
    pub index: usize,
    pub uuid: Uuid,
}

impl Handle {}

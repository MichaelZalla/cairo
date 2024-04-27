use serde::{Deserialize, Serialize};

use crate::serde::PostDeserialize;

use super::{graph::SceneGraph, resources::SceneResources};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneContext {
    pub resources: SceneResources,
    pub scenes: Vec<SceneGraph>,
}

impl PostDeserialize for SceneContext {
    fn post_deserialize(&mut self) {
        self.resources.post_deserialize();

        for scene in &mut self.scenes {
            scene.post_deserialize();
        }
    }
}

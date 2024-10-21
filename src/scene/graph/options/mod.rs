use serde::{Deserialize, Serialize};

use crate::resource::handle::Handle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGraphRenderOptions {
    pub draw_lights: bool,
    pub draw_cameras: bool,
    pub draw_shadow_map_cameras: bool,
    pub camera: Option<Handle>,
}

impl Default for SceneGraphRenderOptions {
    fn default() -> Self {
        Self {
            draw_lights: cfg!(debug_assertions),
            draw_cameras: cfg!(debug_assertions),
            draw_shadow_map_cameras: false,
            camera: None,
        }
    }
}

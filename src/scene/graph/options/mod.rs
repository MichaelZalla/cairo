use serde::{Deserialize, Serialize};

use crate::resource::handle::Handle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGraphRenderOptions {
    pub is_shadow_map_render: bool,
    pub draw_lights: bool,
    pub draw_cameras: bool,
    pub draw_shadow_map_cameras: bool,
    pub camera: Option<Handle>,
}

impl Default for SceneGraphRenderOptions {
    fn default() -> Self {
        Self {
            is_shadow_map_render: false,
            draw_lights: false,
            draw_cameras: cfg!(debug_assertions),
            draw_shadow_map_cameras: false,
            camera: None,
        }
    }
}

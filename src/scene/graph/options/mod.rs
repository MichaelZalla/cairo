use serde::{Deserialize, Serialize};

use crate::resource::handle::Handle;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SceneGraphRenderOptions {
    pub draw_lights: bool,
    pub draw_cameras: bool,
    pub draw_shadow_map_cameras: bool,
    pub camera: Option<Handle>,
}

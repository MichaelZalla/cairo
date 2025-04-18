use sdl2::keyboard::Keycode;

use serde::{Deserialize, Serialize};

use crate::{device::keyboard::KeyboardState, resource::handle::Handle};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SceneGraphRenderOptions {
    pub is_shadow_map_render: bool,
    pub draw_lights: bool,
    pub draw_cameras: bool,
    pub draw_shadow_map_cameras: bool,
    pub draw_node_labels: bool,
    pub camera: Option<Handle>,
}

impl SceneGraphRenderOptions {
    pub fn update(&mut self, keyboard_state: &mut KeyboardState) {
        if keyboard_state.newly_pressed_keycodes.contains(&Keycode::F) {
            self.draw_lights = !self.draw_lights;
            self.draw_cameras = !self.draw_cameras;
        }
    }
}

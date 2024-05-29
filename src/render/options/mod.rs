use sdl2::keyboard::Keycode;

use crate::{
    color::{self, Color},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    render::culling::{FaceCullingReject, FaceCullingWindingOrder},
};

use super::culling::FaceCullingStrategy;

pub mod shader;

#[derive(Debug, Copy, Clone)]
pub struct RenderOptions {
    pub wireframe_color: Color,
    pub do_wireframe: bool,
    pub do_rasterized_geometry: bool,
    pub do_lighting: bool,
    pub do_deferred_lighting: bool,
    pub do_bloom: bool,
    pub do_visualize_normals: bool,
    pub face_culling_strategy: FaceCullingStrategy,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            wireframe_color: color::WHITE,
            do_wireframe: false,
            do_rasterized_geometry: true,
            do_lighting: true,
            do_deferred_lighting: true,
            do_bloom: false,
            do_visualize_normals: false,
            face_culling_strategy: Default::default(),
        }
    }
}

impl RenderOptions {
    pub fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Num1 { .. } => {
                    self.do_wireframe = !self.do_wireframe;

                    println!(
                        "Wireframe: {}",
                        if self.do_wireframe { "On" } else { "Off" }
                    );
                }
                Keycode::Num2 { .. } => {
                    self.do_rasterized_geometry = !self.do_rasterized_geometry;

                    println!(
                        "Rasterized geometry: {}",
                        if self.do_rasterized_geometry {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                Keycode::Num3 { .. } => {
                    self.do_lighting = !self.do_lighting;

                    println!("Lighting: {}", if self.do_lighting { "On" } else { "Off" });
                }
                Keycode::Num4 { .. } => {
                    self.do_deferred_lighting = !self.do_deferred_lighting;

                    println!(
                        "Deferred lighting: {}",
                        if self.do_deferred_lighting {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                Keycode::Num5 { .. } => {
                    self.do_visualize_normals = !self.do_visualize_normals;

                    println!(
                        "Visualize normals: {}",
                        if self.do_visualize_normals {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                Keycode::Num7 { .. } => {
                    self.do_bloom = !self.do_bloom;

                    println!("Bloom pass: {}", if self.do_bloom { "On" } else { "Off" });
                }
                Keycode::Num8 { .. } => {
                    // Cycle culling reject settings.

                    self.face_culling_strategy.reject = match self.face_culling_strategy.reject {
                        FaceCullingReject::None => FaceCullingReject::Backfaces,
                        FaceCullingReject::Backfaces => FaceCullingReject::Frontfaces,
                        FaceCullingReject::Frontfaces => FaceCullingReject::None,
                    };

                    println!(
                        "Face culling reject: {:?}",
                        self.face_culling_strategy.reject
                    );
                }
                Keycode::Num9 { .. } => {
                    // Cycle window orders.

                    self.face_culling_strategy.winding_order =
                        match self.face_culling_strategy.winding_order {
                            FaceCullingWindingOrder::Clockwise => {
                                FaceCullingWindingOrder::CounterClockwise
                            }
                            FaceCullingWindingOrder::CounterClockwise => {
                                FaceCullingWindingOrder::Clockwise
                            }
                        };

                    println!(
                        "Face culling window order: {:?}",
                        self.face_culling_strategy.winding_order
                    );
                }
                _ => {}
            }
        }

        if game_controller_state.buttons.x {
            self.do_wireframe = !self.do_wireframe;
        } else if game_controller_state.buttons.y {
            self.do_visualize_normals = !self.do_visualize_normals;
        }
    }
}

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
    pub do_rasterization: bool,
    pub do_lighting: bool,
    pub do_deferred_lighting: bool,
    pub do_bloom: bool,
    pub face_culling_strategy: FaceCullingStrategy,
    pub draw_wireframe: bool,
    // User debug
    pub wireframe_color: Color,
    pub draw_normals: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            do_rasterization: true,
            do_lighting: true,
            do_deferred_lighting: true,
            do_bloom: false,
            face_culling_strategy: Default::default(),
            draw_wireframe: false,
            // User debug
            wireframe_color: color::WHITE,
            draw_normals: false,
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
                (Keycode::Num1, _) => {
                    self.do_rasterization = !self.do_rasterization;

                    println!(
                        "Rasterization: {}",
                        if self.do_rasterization { "On" } else { "Off" }
                    );
                }
                (Keycode::Num2, _) => {
                    self.do_lighting = !self.do_lighting;

                    println!("Lighting: {}", if self.do_lighting { "On" } else { "Off" });
                }
                (Keycode::Num3, _) => {
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
                (Keycode::Num4, _) => {
                    self.do_bloom = !self.do_bloom;

                    println!("Bloom pass: {}", if self.do_bloom { "On" } else { "Off" });
                }
                (Keycode::Num5, _) => {
                    // Cycle culling reject settings.

                    self.face_culling_strategy.reject = match self.face_culling_strategy.reject {
                        FaceCullingReject::None => FaceCullingReject::Backfaces,
                        FaceCullingReject::Backfaces => FaceCullingReject::Frontfaces,
                        FaceCullingReject::Frontfaces => FaceCullingReject::None,
                    };

                    println!(
                        "Face culling - Reject: {:?}",
                        self.face_culling_strategy.reject
                    );
                }
                (Keycode::Num6, _) => {
                    // Cycle winding order.

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
                        "Face culling - Winding order: {:?}",
                        self.face_culling_strategy.winding_order
                    );
                }
                (Keycode::Num7, _) => {
                    self.draw_wireframe = !self.draw_wireframe;

                    println!(
                        "Draw wireframe: {}",
                        if self.draw_wireframe { "On" } else { "Off" }
                    );
                }
                (Keycode::Num8, _) => {
                    self.draw_normals = !self.draw_normals;

                    println!(
                        "Draw normals: {}",
                        if self.draw_normals { "On" } else { "Off" }
                    );
                }
                _ => {}
            }
        }

        if game_controller_state.buttons.x {
            self.draw_wireframe = !self.draw_wireframe;
        } else if game_controller_state.buttons.y {
            self.draw_normals = !self.draw_normals;
        }
    }
}

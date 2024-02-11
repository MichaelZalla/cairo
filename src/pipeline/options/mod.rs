use sdl2::keyboard::Keycode;

use crate::{
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
};

#[derive(Default, Debug, Copy, Clone)]
pub enum PipelineFaceCullingWindingOrder {
    Clockwise,
    #[default]
    CounterClockwise,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum PipelineFaceCullingReject {
    None,
    #[default]
    Backfaces,
    Frontfaces,
}
#[derive(Default, Debug, Copy, Clone)]
pub struct PipelineFaceCullingStrategy {
    pub reject: PipelineFaceCullingReject,
    pub window_order: PipelineFaceCullingWindingOrder,
}

#[derive(Debug, Copy, Clone)]
pub struct PipelineOptions {
    pub wireframe_color: Color,
    pub do_wireframe: bool,
    pub do_rasterized_geometry: bool,
    pub do_lighting: bool,
    pub do_visualize_normals: bool,
    pub face_culling_strategy: PipelineFaceCullingStrategy,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            wireframe_color: color::WHITE,
            do_wireframe: false,
            do_rasterized_geometry: true,
            do_lighting: true,
            do_visualize_normals: false,
            face_culling_strategy: Default::default(),
        }
    }
}

impl PipelineOptions {
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
                Keycode::Num5 { .. } => {
                    // Cycle culling reject settings.

                    self.face_culling_strategy.reject = match self.face_culling_strategy.reject {
                        PipelineFaceCullingReject::None => PipelineFaceCullingReject::Backfaces,
                        PipelineFaceCullingReject::Backfaces => {
                            PipelineFaceCullingReject::Frontfaces
                        }
                        PipelineFaceCullingReject::Frontfaces => PipelineFaceCullingReject::None,
                    };

                    println!(
                        "Face culling reject: {:?}",
                        self.face_culling_strategy.reject
                    );
                }
                Keycode::Num6 { .. } => {
                    // Cycle window orders.

                    self.face_culling_strategy.window_order =
                        match self.face_culling_strategy.window_order {
                            PipelineFaceCullingWindingOrder::Clockwise => {
                                PipelineFaceCullingWindingOrder::CounterClockwise
                            }
                            PipelineFaceCullingWindingOrder::CounterClockwise => {
                                PipelineFaceCullingWindingOrder::Clockwise
                            }
                        };

                    println!(
                        "Face culling window order: {:?}",
                        self.face_culling_strategy.window_order
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

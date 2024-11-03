use bitmask::bitmask;

use serde::{Deserialize, Serialize};

use sdl2::keyboard::Keycode;

use crate::{
    device::keyboard::KeyboardState,
    render::culling::{FaceCullingReject, FaceCullingWindingOrder},
    resource::handle::Handle,
    vec::vec3::{self, Vec3},
};

use rasterizer::RasterizerOptions;

pub mod rasterizer;
pub mod shader;

bitmask! {
    #[derive(Debug, Serialize, Deserialize)]
    pub mask RenderPassMask: u32 where flags RenderPassFlag {
        Null = 0,
        Rasterization = (1 << 0),
        Lighting = (1 << 1),
        DeferredLighting = (1 << 2),
        Bloom = (1 << 3),
    }
}

impl Default for RenderPassMask {
    fn default() -> Self {
        RenderPassFlag::Rasterization | RenderPassFlag::Lighting | RenderPassFlag::DeferredLighting
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderOptions {
    pub render_pass_flags: RenderPassMask,
    pub bloom_dirt_mask_handle: Option<Handle>,
    pub rasterizer_options: RasterizerOptions,
    // User debug
    pub draw_wireframe: bool,
    pub wireframe_color: Vec3,
    pub draw_normals: bool,
    pub draw_normals_scale: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            render_pass_flags: Default::default(),
            bloom_dirt_mask_handle: None,
            rasterizer_options: Default::default(),
            // User debug
            draw_wireframe: false,
            // User debug
            wireframe_color: vec3::ONES,
            draw_normals: false,
            draw_normals_scale: 0.05,
        }
    }
}

impl RenderOptions {
    pub fn update(&mut self, keyboard_state: &KeyboardState) {
        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                (Keycode::Num1, _) => {
                    self.render_pass_flags ^= RenderPassFlag::Rasterization;

                    println!(
                        "Rasterization pass: {}",
                        if self
                            .render_pass_flags
                            .contains(RenderPassFlag::Rasterization)
                        {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                (Keycode::Num2, _) => {
                    self.render_pass_flags ^= RenderPassFlag::Lighting;

                    println!(
                        "Lighting pass: {}",
                        if self.render_pass_flags.contains(RenderPassFlag::Lighting) {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                (Keycode::Num3, _) => {
                    self.render_pass_flags ^= RenderPassFlag::DeferredLighting;

                    println!(
                        "Deferred lighting pass: {}",
                        if self
                            .render_pass_flags
                            .contains(RenderPassFlag::DeferredLighting)
                        {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                (Keycode::Num4, _) => {
                    self.render_pass_flags ^= RenderPassFlag::Bloom;

                    println!(
                        "Bloom pass: {}",
                        if self.render_pass_flags.contains(RenderPassFlag::Bloom) {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                (Keycode::Num5, _) => {
                    // Cycle culling reject settings.

                    self.rasterizer_options.face_culling_strategy.reject =
                        match self.rasterizer_options.face_culling_strategy.reject {
                            FaceCullingReject::None => FaceCullingReject::Backfaces,
                            FaceCullingReject::Backfaces => FaceCullingReject::Frontfaces,
                            FaceCullingReject::Frontfaces => FaceCullingReject::None,
                        };

                    println!(
                        "Face culling - Reject: {:?}",
                        self.rasterizer_options.face_culling_strategy.reject
                    );
                }
                (Keycode::Num6, _) => {
                    // Cycle winding order.

                    self.rasterizer_options.face_culling_strategy.winding_order =
                        match self.rasterizer_options.face_culling_strategy.winding_order {
                            FaceCullingWindingOrder::Clockwise => {
                                FaceCullingWindingOrder::CounterClockwise
                            }
                            FaceCullingWindingOrder::CounterClockwise => {
                                FaceCullingWindingOrder::Clockwise
                            }
                        };

                    println!(
                        "Face culling - Winding order: {:?}",
                        self.rasterizer_options.face_culling_strategy.winding_order
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
    }
}

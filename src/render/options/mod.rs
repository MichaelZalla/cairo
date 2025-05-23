use bitmask::bitmask;

use serde::{Deserialize, Serialize};

use sdl2::keyboard::Keycode;

use crate::{
    device::keyboard::KeyboardState,
    render::culling::FaceCullingReject,
    resource::handle::Handle,
    vec::vec3::{self, Vec3},
};

use rasterizer::RasterizerOptions;
use tone_mapping::{ToneMappingOperator, TONE_MAPPING_OPERATORS};

pub mod rasterizer;
pub mod shader;
pub mod tone_mapping;

bitmask! {
    #[derive(Debug, Serialize, Deserialize)]
    pub mask RenderPassMask: u32 where flags RenderPassFlag {
        Null = 0,
        Rasterization = (1 << 0),
        Lighting = (1 << 1),
        DeferredLighting = (1 << 2),
        Bloom = (1 << 3),
        Ssao = (1 << 4),
        SsaoBlur = (1 << 5),
        ToneMapping = (1 << 6),
    }
}

impl Default for RenderPassMask {
    fn default() -> Self {
        RenderPassFlag::Rasterization
            | RenderPassFlag::Lighting
            | RenderPassFlag::DeferredLighting
            | RenderPassFlag::ToneMapping
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderOptions {
    pub render_pass_flags: RenderPassMask,
    pub bloom_dirt_mask_handle: Option<Handle>,
    pub rasterizer_options: RasterizerOptions,
    pub tone_mapping: ToneMappingOperator,
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
            tone_mapping: Default::default(),
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
        for keycode in keyboard_state.newly_pressed_keycodes.iter() {
            match *keycode {
                Keycode::Num1 => {
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
                Keycode::Num2 => {
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
                Keycode::Num3 => {
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
                Keycode::Num4 => {
                    self.render_pass_flags ^= RenderPassFlag::Ssao;

                    if self.render_pass_flags.contains(RenderPassFlag::Ssao) {
                        self.render_pass_flags.set(RenderPassFlag::SsaoBlur);
                    } else {
                        self.render_pass_flags.unset(RenderPassFlag::SsaoBlur);
                    }

                    println!(
                        "SSAO pass: {}",
                        if self.render_pass_flags.contains(RenderPassFlag::Ssao) {
                            "On (with blur)"
                        } else {
                            "Off"
                        }
                    );
                }
                Keycode::Num5 => {
                    // Cycle tone-mapping operators.

                    let current_index: usize = self.tone_mapping.try_into().unwrap();

                    let new_index = (current_index + 1).rem_euclid(TONE_MAPPING_OPERATORS.len());

                    self.tone_mapping = TONE_MAPPING_OPERATORS[new_index];

                    println!("Tone mapping: {}", self.tone_mapping);
                }
                Keycode::Num6 => {
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
                Keycode::Num7 => {
                    self.draw_wireframe = !self.draw_wireframe;

                    println!(
                        "Draw wireframe: {}",
                        if self.draw_wireframe { "On" } else { "Off" }
                    );
                }
                Keycode::Num8 => {
                    self.draw_normals = !self.draw_normals;

                    println!(
                        "Draw normals: {}",
                        if self.draw_normals { "On" } else { "Off" }
                    );
                }
                Keycode::H => {
                    self.rasterizer_options.vertex_snapping =
                        !self.rasterizer_options.vertex_snapping;

                    println!(
                        "Vertex snapping: {}",
                        if self.rasterizer_options.vertex_snapping {
                            "On"
                        } else {
                            "Off"
                        }
                    );
                }
                _ => {}
            }
        }
    }
}

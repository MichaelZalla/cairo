use std::fmt;

use crate::{
    color::{self, Color},
    context::ApplicationRenderingContext,
    texture::sample::sample_nearest,
    vec::{vec2::Vec2, vec4::Vec4},
};

use super::TextureMap;

static SIDES: usize = 6;

#[derive(Copy, Clone)]
enum Side {
    FRONT = 0,
    BACK = 1,
    TOP = 2,
    BOTTOM = 3,
    LEFT = 4,
    RIGHT = 5,
}

impl fmt::Display for Side {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Side::FRONT => "FRONT",
            Side::BACK => "BACK",
            Side::TOP => "TOP",
            Side::BOTTOM => "BOTTOM",
            Side::LEFT => "LEFT",
            Side::RIGHT => "RIGHT",
        };

        writeln!(v, "Side (\"{}\")", repr)
    }
}

pub struct CubeMap {
    sides: [TextureMap; 6],
}

impl CubeMap {
    pub fn new(texture_paths: [&str; 6]) -> Self {
        Self {
            sides: [
                TextureMap::new(texture_paths[Side::FRONT as usize]),
                TextureMap::new(texture_paths[Side::BACK as usize]),
                TextureMap::new(texture_paths[Side::TOP as usize]),
                TextureMap::new(texture_paths[Side::BOTTOM as usize]),
                TextureMap::new(texture_paths[Side::LEFT as usize]),
                TextureMap::new(texture_paths[Side::RIGHT as usize]),
            ],
        }
    }

    pub fn load(&mut self, rendering_context: &ApplicationRenderingContext) -> Result<(), String> {
        for index in 0..SIDES {
            self.sides[index].load(rendering_context)?;
        }

        return Ok(());
    }

    pub fn sample(&self, direction: &Vec4) -> Color {
        let absolute = direction.abs();

        let side: Side;

        let uv_scaling_factor: f32;

        let mut uv: Vec2;

        if absolute.x >= absolute.y && absolute.x >= absolute.z {
            // X has the greatest magnitude
            side = if direction.x >= 0.0 {
                Side::RIGHT
            } else {
                Side::LEFT
            };

            uv_scaling_factor = 0.5 / absolute.x;

            uv = Vec2 {
                x: if direction.x < 0.0 {
                    -direction.z
                } else {
                    direction.z
                },
                y: direction.y,
                z: 0.0,
            };
        } else if absolute.y >= absolute.z {
            // Y has the greatest magnitude
            side = if direction.y >= 0.0 {
                Side::TOP
            } else {
                Side::BOTTOM
            };

            uv_scaling_factor = 0.5 / absolute.y;

            uv = Vec2 {
                x: -direction.x,
                y: if direction.y < 0.0 {
                    direction.z
                } else {
                    -direction.z
                },
                z: 0.0,
            };
        } else {
            // Z has the greatest magnitude
            side = if direction.z >= 0.0 {
                Side::FRONT
            } else {
                Side::BACK
            };

            uv_scaling_factor = 0.5 / absolute.z;

            uv = Vec2 {
                x: if direction.z < 0.0 {
                    direction.x
                } else {
                    -direction.x
                },
                y: direction.y,
                z: 0.0,
            };
        }

        uv *= uv_scaling_factor;
        uv.x += 0.5;
        uv.y += 0.5;

        let map: &TextureMap = &self.sides[side as usize];

        if !map.is_loaded {
            static COLORS: [Color; 6] = [
                color::BLUE,
                color::RED,
                color::WHITE,
                color::BLACK,
                color::GREEN,
                color::YELLOW,
            ];

            return COLORS[side as usize];
        }

        let (r, g, b) = sample_nearest(uv, &map, None);

        Color::rgb(r, g, b)
    }
}

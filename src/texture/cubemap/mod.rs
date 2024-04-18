use std::fmt;

use crate::{
    app::context::ApplicationRenderingContext,
    buffer::Buffer2D,
    color::{self, Color},
    texture::sample::sample_nearest,
    vec::{vec2::Vec2, vec4::Vec4},
};

use super::map::{TextureMap, TextureMapStorageFormat};

static SIDES: usize = 6;

#[derive(Copy, Clone, Debug)]
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
    is_cross: bool,
    sides: [TextureMap; 6],
}

impl CubeMap {
    pub fn new(texture_paths: [&str; 6], storage_format: TextureMapStorageFormat) -> Self {
        Self {
            is_cross: false,
            sides: [
                TextureMap::new(texture_paths[Side::FRONT as usize], storage_format),
                TextureMap::new(texture_paths[Side::BACK as usize], storage_format),
                TextureMap::new(texture_paths[Side::TOP as usize], storage_format),
                TextureMap::new(texture_paths[Side::BOTTOM as usize], storage_format),
                TextureMap::new(texture_paths[Side::LEFT as usize], storage_format),
                TextureMap::new(texture_paths[Side::RIGHT as usize], storage_format),
            ],
        }
    }

    pub fn cross(texture_path: &str, storage_format: TextureMapStorageFormat) -> Self {
        Self {
            is_cross: true,
            sides: [
                TextureMap::new(texture_path, storage_format),
                TextureMap::new(texture_path, storage_format),
                TextureMap::new(texture_path, storage_format),
                TextureMap::new(texture_path, storage_format),
                TextureMap::new(texture_path, storage_format),
                TextureMap::new(texture_path, storage_format),
            ],
        }
    }

    pub fn load(&mut self, rendering_context: &ApplicationRenderingContext) -> Result<(), String> {
        if self.is_cross {
            // Read in the horizontal or vertical cross texture

            let mut map = TextureMap::new(
                &self.sides[0].info.filepath,
                self.sides[0].info.storage_format,
            );

            map.load(rendering_context).unwrap();

            let is_horizontal = map.width > map.height;

            let dimension = if is_horizontal {
                map.width / 4
            } else {
                map.height / 4
            };

            let cross_buffer = &map.levels[0];

            for (side_index, side_map) in self.sides.iter_mut().enumerate() {
                let side = match side_index {
                    0 => Side::FRONT,
                    1 => Side::BACK,
                    2 => Side::TOP,
                    3 => Side::BOTTOM,
                    4 => Side::LEFT,
                    _ => Side::RIGHT,
                };

                side_map.width = dimension;
                side_map.height = dimension;

                let block_coordinate: (u32, u32);

                match side {
                    Side::FRONT => {
                        block_coordinate = (1, 1);
                    }
                    Side::BACK => {
                        block_coordinate = if is_horizontal { (3, 1) } else { (1, 3) };
                    }
                    Side::TOP => {
                        block_coordinate = (1, 0);
                    }
                    Side::BOTTOM => {
                        block_coordinate = (1, 2);
                    }
                    Side::LEFT => {
                        block_coordinate = (0, 1);
                    }
                    Side::RIGHT => {
                        block_coordinate = (2, 1);
                    }
                }

                let block_pixel_coordinate = (
                    block_coordinate.0 * dimension,
                    block_coordinate.1 * dimension,
                );

                // Blit the corresponding pixels into this texture map's root level.

                let mut bytes: Vec<u8> = vec![];

                let bytes_per_pixel = side_map.get_bytes_per_pixel();

                let new_len = dimension as usize * dimension as usize * bytes_per_pixel;

                bytes.resize(new_len, 0);

                assert!(bytes.len() == new_len);

                for global_y in block_pixel_coordinate.1..(block_pixel_coordinate.1 + dimension) {
                    for global_x in block_pixel_coordinate.0..(block_pixel_coordinate.0 + dimension)
                    {
                        let global_pixel_index = ((global_y * map.width) as usize
                            * bytes_per_pixel)
                            + global_x as usize * bytes_per_pixel;

                        let mut local_x = global_x - block_pixel_coordinate.0;
                        let mut local_y = global_y - block_pixel_coordinate.1;

                        if side_index == Side::BACK as usize && !is_horizontal {
                            // Flip back texture data
                            local_x = dimension - local_x - 1;
                            local_y = dimension - local_y - 1;
                        }

                        let local_pixel_index = ((local_y * dimension) as usize * bytes_per_pixel)
                            + local_x as usize * bytes_per_pixel;

                        bytes[local_pixel_index] = cross_buffer.data[global_pixel_index];

                        match side_map.info.storage_format {
                            TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                                bytes[local_pixel_index + 1] =
                                    cross_buffer.data[global_pixel_index + 1];
                                bytes[local_pixel_index + 2] =
                                    cross_buffer.data[global_pixel_index + 2];
                            }
                            TextureMapStorageFormat::Index8(_target_channel) => (),
                        }
                    }
                }

                let buffer = Buffer2D::from_data(dimension, dimension, bytes);

                side_map.levels.push(buffer);

                side_map.is_loaded = true;
            }
        } else {
            for index in 0..SIDES {
                self.sides[index].load(rendering_context)?;
            }
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
                x: direction.x,
                y: if direction.y < 0.0 {
                    -direction.z
                } else {
                    direction.z
                },
                z: 0.0,
            };
        } else {
            // Z has the greatest magnitude
            side = if direction.z >= 0.0 {
                Side::BACK
            } else {
                Side::FRONT
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

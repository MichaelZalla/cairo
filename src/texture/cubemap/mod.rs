use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use crate::{
    app::context::ApplicationRenderingContext,
    buffer::Buffer2D,
    color::{self, Color},
    serde::PostDeserialize,
    texture::{map::TextureBuffer, sample::sample_nearest_u8},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::{
    map::{TextureMap, TextureMapStorageFormat},
    sample::{sample_nearest_vec3, sample_trilinear_vec3},
};

static SIDES: usize = 6;

#[derive(Copy, Clone, Debug)]
pub enum Side {
    Front = 0,
    Back = 1,
    Top = 2,
    Bottom = 3,
    Left = 4,
    Right = 5,
}

pub static CUBE_MAP_SIDES: [Side; 6] = [
    Side::Front,
    Side::Back,
    Side::Top,
    Side::Bottom,
    Side::Left,
    Side::Right,
];

static CUBEMAP_SIDE_COLORS: [Color; 6] = [
    color::BLUE,
    color::RED,
    color::WHITE,
    color::BLACK,
    color::GREEN,
    color::YELLOW,
];

impl fmt::Display for Side {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Side::Front => "Front",
            Side::Back => "Back",
            Side::Top => "Top",
            Side::Bottom => "Bottom",
            Side::Left => "Left",
            Side::Right => "Right",
        };

        writeln!(v, "Side (\"{}\")", repr)
    }
}

impl Side {
    pub fn get_direction(&self) -> Vec3 {
        match self {
            Side::Front => vec3::FORWARD,
            Side::Back => vec3::FORWARD * -1.0,
            Side::Top => Vec3 {
                x: -0.0,
                y: 1.0,
                z: 0.0001,
            },
            Side::Bottom => Vec3 {
                x: -0.0,
                y: -1.0,
                z: 0.0001,
            },
            Side::Left => vec3::RIGHT * -1.0,
            Side::Right => vec3::RIGHT,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CubeMap<T: Default + Debug + Copy + PartialEq = u8> {
    is_cross: bool,
    pub sides: [TextureMap<T>; 6],
}

impl<T: Default + Debug + Copy + PartialEq> PostDeserialize for CubeMap<T> {
    fn post_deserialize(&mut self) {
        for side in self.sides.iter_mut() {
            side.post_deserialize();
        }
    }
}

impl<T: Default + Debug + Copy + PartialEq> CubeMap<T> {
    pub fn new(texture_paths: [&str; 6], storage_format: TextureMapStorageFormat) -> Self {
        Self {
            is_cross: false,
            sides: [
                TextureMap::new(texture_paths[Side::Front as usize], storage_format),
                TextureMap::new(texture_paths[Side::Back as usize], storage_format),
                TextureMap::new(texture_paths[Side::Top as usize], storage_format),
                TextureMap::new(texture_paths[Side::Bottom as usize], storage_format),
                TextureMap::new(texture_paths[Side::Left as usize], storage_format),
                TextureMap::new(texture_paths[Side::Right as usize], storage_format),
            ],
        }
    }

    pub fn from_textures(sides: [TextureMap<T>; 6]) -> Self {
        Self {
            is_cross: false,
            sides,
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

    pub fn get_uv_for_direction(&self, direction: &Vec4) -> (Side, Vec2) {
        let absolute = direction.abs();

        let side: Side;

        let uv_scaling_factor: f32;

        let mut uv: Vec2;

        if absolute.x >= absolute.y && absolute.x >= absolute.z {
            // X has the greatest magnitude
            side = if direction.x >= 0.0 {
                Side::Right
            } else {
                Side::Left
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
                Side::Top
            } else {
                Side::Bottom
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
                Side::Back
            } else {
                Side::Front
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

        (side, uv)
    }
}

impl CubeMap<Vec3> {
    pub fn sample_nearest(&self, direction: &Vec4, level_index: Option<usize>) -> Vec3 {
        let (side, uv) = self.get_uv_for_direction(direction);

        let map = &self.sides[side as usize];

        if !map.is_loaded {
            return CUBEMAP_SIDE_COLORS[side as usize].to_vec3();
        }

        sample_nearest_vec3(uv, map, level_index)
    }

    pub fn sample_trilinear(
        &self,
        direction: &Vec4,
        near_level_index: usize,
        far_level_index: usize,
        alpha: f32,
    ) -> Vec3 {
        let (side, uv) = self.get_uv_for_direction(direction);

        let map = &self.sides[side as usize];

        if !map.is_loaded {
            return CUBEMAP_SIDE_COLORS[side as usize].to_vec3();
        }

        sample_trilinear_vec3(uv, map, near_level_index, far_level_index, alpha)
    }
}

impl CubeMap {
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
                    0 => Side::Front,
                    1 => Side::Back,
                    2 => Side::Top,
                    3 => Side::Bottom,
                    4 => Side::Left,
                    _ => Side::Right,
                };

                side_map.width = dimension;
                side_map.height = dimension;

                let block_coordinate = match side {
                    Side::Front => (1, 1),
                    Side::Back => {
                        if is_horizontal {
                            (3, 1)
                        } else {
                            (1, 3)
                        }
                    }
                    Side::Top => (1, 0),
                    Side::Bottom => (1, 2),
                    Side::Left => (0, 1),
                    Side::Right => (2, 1),
                };

                let block_pixel_coordinate = (
                    block_coordinate.0 * dimension,
                    block_coordinate.1 * dimension,
                );

                // Blit the corresponding pixels into this texture map's root level.

                let mut bytes: Vec<u8> = vec![];

                let buffer_samples_per_pixel = side_map.get_buffer_samples_per_pixel();

                let new_len = dimension as usize * dimension as usize * buffer_samples_per_pixel;

                bytes.resize(new_len, 0);

                assert!(bytes.len() == new_len);

                for global_y in block_pixel_coordinate.1..(block_pixel_coordinate.1 + dimension) {
                    for global_x in block_pixel_coordinate.0..(block_pixel_coordinate.0 + dimension)
                    {
                        let global_pixel_index = ((global_y * map.width) as usize
                            * buffer_samples_per_pixel)
                            + global_x as usize * buffer_samples_per_pixel;

                        let mut local_x = global_x - block_pixel_coordinate.0;
                        let mut local_y = global_y - block_pixel_coordinate.1;

                        if side_index == Side::Back as usize && !is_horizontal {
                            // Flip back texture data
                            local_x = dimension - local_x - 1;
                            local_y = dimension - local_y - 1;
                        }

                        let local_pixel_index = ((local_y * dimension) as usize
                            * buffer_samples_per_pixel)
                            + local_x as usize * buffer_samples_per_pixel;

                        bytes[local_pixel_index] = cross_buffer.0.data[global_pixel_index];

                        match side_map.info.storage_format {
                            TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                                bytes[local_pixel_index + 1] =
                                    cross_buffer.0.data[global_pixel_index + 1];
                                bytes[local_pixel_index + 2] =
                                    cross_buffer.0.data[global_pixel_index + 2];
                            }
                            TextureMapStorageFormat::Index8(_target_channel) => (),
                        }
                    }
                }

                let buffer = Buffer2D::from_data(dimension, dimension, bytes);

                side_map.levels.push(TextureBuffer(buffer));

                side_map.is_loaded = true;
            }
        } else {
            for index in 0..SIDES {
                self.sides[index].load(rendering_context)?;
            }
        }

        Ok(())
    }

    pub fn sample_nearest(&self, direction: &Vec4, level_index: Option<usize>) -> Color {
        let (side, uv) = self.get_uv_for_direction(direction);

        let map = &self.sides[side as usize];

        if !map.is_loaded {
            return CUBEMAP_SIDE_COLORS[side as usize];
        }

        let (r, g, b) = sample_nearest_u8(uv, map, level_index);

        Color::rgb(r, g, b)
    }
}

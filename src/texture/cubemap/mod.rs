use std::{
    cell::RefCell,
    fmt::{self, Debug},
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    app::context::ApplicationRenderingContext,
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::{self, Color},
    render::Renderer,
    scene::{camera::Camera, context::SceneContext},
    serde::PostDeserialize,
    shader::context::ShaderContext,
    texture::{map::TextureBuffer, sample::sample_nearest_u8},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::{
    map::{TextureMap, TextureMapStorageFormat},
    sample::{sample_bilinear_u8, sample_nearest_f32, sample_nearest_vec3, sample_trilinear_vec3},
};

static SIDES: usize = 6;

#[derive(Copy, Clone, Debug)]
pub enum Side {
    Forward = 0,
    Backward = 1,
    Up = 2,
    Down = 3,
    Left = 4,
    Right = 5,
}

pub static CUBE_MAP_SIDES: [Side; 6] = [
    Side::Forward,
    Side::Backward,
    Side::Up,
    Side::Down,
    Side::Left,
    Side::Right,
];

pub static CUBEMAP_SIDE_COLORS: [Color; 6] = [
    // Forward
    color::GREEN,
    // Back
    color::YELLOW,
    // Up
    color::BLUE,
    // Bottom
    color::BLACK,
    // Left
    color::WHITE,
    // Right
    color::RED,
];

impl fmt::Display for Side {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Side::Forward => "Front",
            Side::Backward => "Back",
            Side::Up => "Top",
            Side::Down => "Bottom",
            Side::Left => "Left",
            Side::Right => "Right",
        };

        writeln!(v, "Side (\"{}\")", repr)
    }
}

impl Side {
    pub fn get_index(&self) -> usize {
        match self {
            Side::Forward => 0,
            Side::Backward => 1,
            Side::Up => 2,
            Side::Down => 3,
            Side::Left => 4,
            Side::Right => 5,
        }
    }

    pub fn get_direction(&self) -> Vec3 {
        match self {
            Side::Forward => vec3::FORWARD,
            Side::Backward => vec3::FORWARD * -1.0,
            Side::Up => Vec3 {
                x: -0.0,
                y: 1.0,
                z: 0.0001,
            },
            Side::Down => Vec3 {
                x: -0.0,
                y: -1.0,
                z: 0.0001,
            },
            Side::Left => vec3::RIGHT * -1.0,
            Side::Right => vec3::RIGHT,
        }
    }

    pub fn get_block_coordinate(&self, is_horizontal: bool) -> (u32, u32) {
        match self {
            Side::Forward => (1, 1),
            Side::Backward => {
                if is_horizontal {
                    (3, 1)
                } else {
                    (1, 3)
                }
            }
            Side::Up => (1, 0),
            Side::Down => (1, 2),
            Side::Left => (0, 1),
            Side::Right => (2, 1),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CubeMap<T: Default + Debug + Copy + PartialEq + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> = u8> {
    is_cross: bool,
    pub sides: [TextureMap<T>; 6],
}

impl<
        T: Default
            + Debug
            + Copy
            + PartialEq
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    > From<[TextureMap<T>; 6]> for CubeMap<T>
{
    fn from(sides: [TextureMap<T>; 6]) -> Self {
        Self {
            is_cross: false,
            sides,
        }
    }
}

impl<
        T: Default
            + PartialEq
            + Copy
            + Clone
            + Debug
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    > From<&Framebuffer> for CubeMap<T>
{
    fn from(framebuffer: &Framebuffer) -> Self {
        let cubemap_size = {
            debug_assert_eq!(framebuffer.width, framebuffer.height);

            framebuffer.width
        };

        let texture_map = TextureMap::from_buffer(
            cubemap_size,
            cubemap_size,
            Buffer2D::<T>::new(cubemap_size, cubemap_size, None),
        );

        let mut cubemap: CubeMap<T> = Default::default();

        for side_index in 0..6 {
            cubemap.sides[side_index] = texture_map.clone();
        }

        cubemap
    }
}

impl<
        T: Default
            + Debug
            + Copy
            + PartialEq
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    > PostDeserialize for CubeMap<T>
{
    fn post_deserialize(&mut self) {
        for side in self.sides.iter_mut() {
            side.post_deserialize();
        }
    }
}

impl<
        T: Default
            + Debug
            + Copy
            + PartialEq
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    > CubeMap<T>
{
    pub fn new(texture_paths: [&str; 6], storage_format: TextureMapStorageFormat) -> Self {
        Self {
            is_cross: false,
            sides: [
                TextureMap::new(texture_paths[Side::Forward as usize], storage_format),
                TextureMap::new(texture_paths[Side::Backward as usize], storage_format),
                TextureMap::new(texture_paths[Side::Up as usize], storage_format),
                TextureMap::new(texture_paths[Side::Down as usize], storage_format),
                TextureMap::new(texture_paths[Side::Left as usize], storage_format),
                TextureMap::new(texture_paths[Side::Right as usize], storage_format),
            ],
        }
    }

    pub fn is_cross(&self) -> bool {
        self.is_cross
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
                x: match side {
                    Side::Right => -direction.z,
                    Side::Left => direction.z,
                    _ => panic!(),
                },
                y: direction.y,
                z: 0.0,
            };
        } else if absolute.y >= absolute.z {
            // Y has the greatest magnitude
            side = if direction.y >= 0.0 {
                Side::Up
            } else {
                Side::Down
            };

            uv_scaling_factor = 0.5 / absolute.y;

            uv = Vec2 {
                x: direction.x,
                y: match side {
                    Side::Up => -direction.z,
                    Side::Down => direction.z,
                    _ => panic!(),
                },
                z: 0.0,
            };
        } else {
            // Z has the greatest magnitude
            side = if direction.z >= 0.0 {
                Side::Forward
            } else {
                Side::Backward
            };

            uv_scaling_factor = 0.5 / absolute.z;

            uv = Vec2 {
                x: match side {
                    Side::Forward => direction.x,
                    Side::Backward => -direction.x,
                    _ => panic!(),
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

impl CubeMap<f32> {
    pub fn sample_nearest(&self, direction: &Vec4) -> f32 {
        let (side, uv) = self.get_uv_for_direction(direction);

        let map = &self.sides[side as usize];

        if !map.is_loaded {
            return CUBEMAP_SIDE_COLORS[side as usize].to_vec3().x;
        }

        sample_nearest_f32(uv, map)
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

    pub fn render_scene(
        &mut self,
        mipmap_level: Option<usize>,
        framebuffer_rc: Rc<RefCell<Framebuffer>>,
        scene_context: &SceneContext,
        shader_context_rc: &RefCell<ShaderContext>,
        renderer_rc: &RefCell<dyn Renderer>,
    ) -> Result<(), String> {
        // Render each face of our cubemap.

        let mut cubemap_face_camera =
            Camera::perspective(Default::default(), vec3::FORWARD, 90.0, 1.0);

        for side in CUBE_MAP_SIDES {
            cubemap_face_camera
                .look_vector
                .set_target(side.get_direction());

            {
                let mut shader_context = (*shader_context_rc).borrow_mut();

                cubemap_face_camera.update_shader_context(&mut shader_context);
            }

            // Begin frame

            {
                let mut renderer = renderer_rc.borrow_mut();

                renderer.begin_frame();
            }

            let scene = &scene_context.scenes.borrow()[0];

            scene.render(&scene_context.resources, renderer_rc, None)?;

            // End frame

            {
                let mut renderer = renderer_rc.borrow_mut();

                renderer.end_frame();
            }

            // Blit our framebuffer's color attachment buffer to our cubemap
            // face texture.

            let framebuffer = framebuffer_rc.borrow();

            match &framebuffer.attachments.deferred_hdr {
                Some(hdr_attachment_rc) => {
                    let hdr_buffer = hdr_attachment_rc.borrow();

                    self.sides[side as usize].levels[mipmap_level.unwrap_or(0)] =
                        TextureBuffer::<Vec3>(hdr_buffer.clone());
                }
                None => return Err("Called CubeMap::<Vec3>::render_scene() with a Framebuffer with no HDR attachment!".to_string()),
            }
        }

        Ok(())
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
                    0 => Side::Forward,
                    1 => Side::Backward,
                    2 => Side::Up,
                    3 => Side::Down,
                    4 => Side::Left,
                    _ => Side::Right,
                };

                side_map.width = dimension;
                side_map.height = dimension;

                let block_coordinate = side.get_block_coordinate(is_horizontal);

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

                        if side_index == Side::Backward as usize && !is_horizontal {
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

    pub fn sample_bilinear(&self, direction: &Vec4, level_index: Option<usize>) -> Color {
        let (side, uv) = self.get_uv_for_direction(direction);

        let map = &self.sides[side as usize];

        if !map.is_loaded {
            return CUBEMAP_SIDE_COLORS[side as usize];
        }

        let (r, g, b) = sample_bilinear_u8(uv, map, level_index);

        Color::rgb(r, g, b)
    }
}

use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::{
    app::context::ApplicationRenderingContext, buffer::Buffer2D, color::Color,
    serde::PostDeserialize, vec::vec2::Vec2, vec::vec3::Vec3,
};

use super::{
    get_half_scaled_u8, get_half_scaled_vec3,
    sample::{sample_bilinear_u8, sample_nearest_u8, sample_trilinear_u8, TextureSamplingMethod},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TextureBuffer<T: Default + Debug + Copy + PartialEq = u8>(pub Buffer2D<T>);

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextureMapStorageFormat {
    #[default]
    RGBA32,
    RGB24,
    Index8(usize),
}

impl TextureMapStorageFormat {
    pub fn get_buffer_samples_per_pixel(&self) -> usize {
        match self {
            TextureMapStorageFormat::RGBA32 => 4,
            TextureMapStorageFormat::RGB24 => 3,
            TextureMapStorageFormat::Index8(_target_channel) => 1,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextureMapInfo {
    pub filepath: String,
    pub storage_format: TextureMapStorageFormat,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextureMapWrapping {
    #[default]
    Repeat,
    ClampToEdge,
    ClampToBorder((u8, u8, u8)),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextureMapSamplingOptions {
    pub wrapping: TextureMapWrapping,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextureMap<T: Default + Debug + Copy + PartialEq = u8> {
    pub info: TextureMapInfo,
    #[serde(skip, default)]
    pub is_loaded: bool,
    #[serde(skip, default)]
    pub has_mipmaps_generated: bool,
    #[serde(skip, default)]
    pub width: u32,
    #[serde(skip, default)]
    pub height: u32,
    #[serde(skip, default)]
    pub levels: Vec<TextureBuffer<T>>,
    pub sampling_options: TextureMapSamplingOptions,
}

impl<T: Default + Debug + Copy + PartialEq> From<Buffer2D<T>> for TextureMap<T> {
    fn from(buffer: Buffer2D<T>) -> Self {
        let width = buffer.width;
        let height = buffer.height;

        let buffer_samples_per_pixel = buffer.data.len() as u32 / width / height;

        Self {
            info: TextureMapInfo {
                filepath: "Buffer".to_string(),
                storage_format: if buffer_samples_per_pixel == 4 {
                    TextureMapStorageFormat::RGBA32
                } else if buffer_samples_per_pixel == 3 {
                    TextureMapStorageFormat::RGB24
                } else if buffer_samples_per_pixel == 1 {
                    TextureMapStorageFormat::Index8(0)
                } else {
                    panic!(
                        "Invalid buffer data length {} for buffer size {}x{}!",
                        buffer.data.len(),
                        width,
                        height
                    )
                },
            },
            is_loaded: true,
            has_mipmaps_generated: false,
            width,
            height,
            levels: vec![TextureBuffer(buffer)],
            sampling_options: Default::default(),
        }
    }
}

impl<T: Default + Debug + Copy + PartialEq> PostDeserialize for TextureMap<T> {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl<T: Default + Debug + Copy + PartialEq> TextureMap<T> {
    pub fn new(filepath: &str, storage_format: TextureMapStorageFormat) -> Self {
        Self {
            info: TextureMapInfo {
                filepath: filepath.to_string(),
                storage_format,
            },
            is_loaded: false,
            has_mipmaps_generated: false,
            width: 0,
            height: 0,
            levels: vec![],
            sampling_options: Default::default(),
        }
    }

    pub fn get_aspect_ratio(&self) -> Option<f32> {
        if self.is_loaded {
            Some(self.levels[0].0.width_over_height)
        } else {
            None
        }
    }

    pub fn get_buffer_samples_per_pixel(&self) -> usize {
        self.info.storage_format.get_buffer_samples_per_pixel()
    }

    pub fn validate_for_mipmapping(&mut self) -> Result<(), String> {
        if !self.is_loaded {
            return Err(String::from(
                "Called TextureMap::generate_mipmaps() on an unloaded texture.",
            ));
        }

        if self.has_mipmaps_generated {
            #[cfg(feature = "print_warnings")]
            println!(
                "Called TextureMap::validate_for_mipmapping() on a TextureMap that already has mipmaps created."
            );

            return Ok(());
        }

        // Validate that this texture is suitable for mipmapping.
        let levels = (self.width as f32).log2() + 1.0;

        if self.width != self.height || levels.fract() != 0.0 {
            return Err(String::from("Called TextureMap::validate_for_mipmapping() on a texture whose dimensions do not support mipmapping."));
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        if self.levels.is_empty() {
            self.levels
                .push(TextureBuffer(Buffer2D::new(width, height, None)))
        } else {
            self.levels[0].0.resize(width, height);
        }

        self.levels = vec![self.levels[0].to_owned()];

        self.has_mipmaps_generated = false;
    }

    pub fn map<C>(&mut self, mut callback: C) -> Result<(), String>
    where
        C: FnMut(T, T, T) -> (T, T, T),
    {
        if !self.is_loaded {
            return Err("Called TextureMap::map() on an unloaded texture!".to_string());
        }

        let buffer_samples_per_pixel = self.get_buffer_samples_per_pixel();

        let original_size_buffer = &mut self.levels[0];

        for i in 0..(self.width * self.height) as usize {
            let r = original_size_buffer.0.data[i * buffer_samples_per_pixel];
            let g;
            let b;

            match self.info.storage_format {
                TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                    g = original_size_buffer.0.data[i * buffer_samples_per_pixel + 1];
                    b = original_size_buffer.0.data[i * buffer_samples_per_pixel + 2];
                }
                TextureMapStorageFormat::Index8(_target_channel) => {
                    g = r;
                    b = r;
                }
            }

            let (r_new, g_new, b_new) = callback(r, g, b);

            original_size_buffer.0.data[i * buffer_samples_per_pixel] = r_new;

            match self.info.storage_format {
                TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                    original_size_buffer.0.data[i * buffer_samples_per_pixel + 1] = g_new;
                    original_size_buffer.0.data[i * buffer_samples_per_pixel + 2] = b_new;
                }
                TextureMapStorageFormat::Index8(_target_channel) => (),
            }
        }

        Ok(())
    }
}

impl TextureMap {
    pub fn from_alpha_channel(filepath: &str) -> Self {
        let target_channel = PixelFormatEnum::RGBA32.byte_size_per_pixel() - 1;

        Self::new(filepath, TextureMapStorageFormat::Index8(target_channel))
    }

    pub fn load(&mut self, rendering_context: &ApplicationRenderingContext) -> Result<(), String> {
        if self.is_loaded {
            return Ok(());
        }

        // Load the map's native-sized pixel data on-demand.

        let mut canvas = rendering_context.canvas.borrow_mut();

        let texture_creator = canvas.texture_creator();

        let static_texture = texture_creator
            .load_texture(self.info.filepath.clone())
            .unwrap();

        let texture_attrs = static_texture.query();

        self.width = texture_attrs.width;
        self.height = texture_attrs.height;

        let mut target_texture = texture_creator
            .create_texture(
                PixelFormatEnum::RGBA32,
                TextureAccess::Target,
                self.width,
                self.height,
            )
            .unwrap();

        canvas
            .with_texture_canvas(&mut target_texture, |texture_canvas| {
                texture_canvas.copy(&static_texture, None, None).unwrap();

                let sdl_read_pixel_format = match self.info.storage_format {
                    TextureMapStorageFormat::RGBA32 => {
                        debug_assert!(
                            TextureMapStorageFormat::RGBA32.get_buffer_samples_per_pixel()
                                == PixelFormatEnum::RGBA32.byte_size_per_pixel()
                        );
                        PixelFormatEnum::RGBA32
                    }
                    TextureMapStorageFormat::RGB24 => {
                        debug_assert!(
                            TextureMapStorageFormat::RGB24.get_buffer_samples_per_pixel()
                                == PixelFormatEnum::RGB24.byte_size_per_pixel()
                        );
                        PixelFormatEnum::RGB24
                    }
                    // Err: "Indexed pixel formats not supported"
                    TextureMapStorageFormat::Index8(_target_channel) => PixelFormatEnum::RGBA32,
                };

                let bytes_per_src_pixel = sdl_read_pixel_format.byte_size_per_pixel();
                let bytes_per_dest_pixel = self.info.storage_format.get_buffer_samples_per_pixel();

                let pixels = texture_canvas
                    .read_pixels(None, sdl_read_pixel_format)
                    .unwrap();

                let pixels_bytes = pixels.len();

                debug_assert!(
                    pixels_bytes as u32 == self.width * self.height * bytes_per_src_pixel as u32,
                    "Invalid `pixels` length {} for width {}, height {}, and bpp {}!",
                    pixels_bytes,
                    self.width,
                    self.height,
                    bytes_per_src_pixel
                );

                let mut original_size_bytes: Vec<u8> = vec![];

                match self.info.storage_format {
                    TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                        original_size_bytes.resize(pixels_bytes, 0);

                        original_size_bytes.copy_from_slice(pixels.as_slice());
                    }
                    TextureMapStorageFormat::Index8(target_channel) => {
                        original_size_bytes.resize(pixels_bytes / bytes_per_src_pixel, 0);

                        if target_channel >= bytes_per_src_pixel {
                            panic!("Invalid channel offset ({}) for texture map with Index8 storage format!", target_channel);
                        }

                        for i in 0..pixels_bytes / bytes_per_src_pixel {
                            original_size_bytes[i] = pixels[i * bytes_per_src_pixel + target_channel];
                        }
                    }
                }

                let buffer = Buffer2D::from_data(self.width, self.height, original_size_bytes);

                debug_assert!(
                    buffer.data.len()
                        == (buffer.width * buffer.height) as usize * bytes_per_dest_pixel,
                    "Invalid buffer data length {} for width {}, height {}, and bpp {}!",
                    buffer.data.len(),
                    buffer.width,
                    buffer.height,
                    bytes_per_dest_pixel
                );

                self.levels.push(TextureBuffer(buffer));
            })
            .unwrap();

        self.is_loaded = true;

        Ok(())
    }

    pub fn generate_mipmaps(&mut self) -> Result<(), String> {
        self.validate_for_mipmapping()?;

        let levels = (self.width as f32).log2() + 1.0;

        // Generate each level of our mipmapped texture
        for level_index in 1..levels as usize {
            let dimension = self.width / 2_u32.pow(level_index as u32);

            let last = self.levels.last().unwrap();

            let bytes = get_half_scaled_u8(dimension, &last.0);

            self.levels.push(TextureBuffer(Buffer2D::from_data(
                dimension, dimension, bytes,
            )));
        }

        self.has_mipmaps_generated = true;

        Ok(())
    }

    fn get_near_far_alpha(&self, width: u32) -> (usize, Option<usize>, Option<f32>) {
        if !self.has_mipmaps_generated {
            return (0, None, None);
        }

        let mut near_level_index = 0;
        let mut far_level_index = 0;

        while self.levels[far_level_index].0.width >= width
            && far_level_index < self.levels.len() - 1
        {
            far_level_index += 1;

            near_level_index = far_level_index - 1;
        }

        let near = near_level_index;

        let far = if far_level_index != near_level_index {
            Some(far_level_index)
        } else {
            None
        };

        let alpha = if far_level_index > near_level_index {
            let near_width = self.levels[near_level_index].0.width;
            let far_width = self.levels[far_level_index].0.width;

            // 1.0 - (req - near) / (far - near)

            Some(1.0 - (width - far_width) as f32 / (near_width - far_width) as f32)
        } else {
            None
        };

        (near, far, alpha)
    }

    pub fn blit_resized(
        &self,
        top: u32,
        left: u32,
        width: u32,
        height: u32,
        sampling_method: TextureSamplingMethod,
        target: &mut Buffer2D,
    ) {
        // Blits a scaled version of `self` to `target`.

        match sampling_method {
            TextureSamplingMethod::NearestNeighbor => {
                self.blit_resized_nearest(top, left, width, height, 0, target);
            }
            TextureSamplingMethod::Bilinear => {
                let (near_index, _, _) = self.get_near_far_alpha(width);

                self.blit_resized_bilinear(top, left, width, height, near_index, target);
            }
            TextureSamplingMethod::Trilinear => match self.get_near_far_alpha(width) {
                (near_index, None, _) => {
                    self.blit_resized_bilinear(top, left, width, height, near_index, target);
                }
                (near_index, Some(far_index), Some(alpha)) => {
                    self.blit_resized_trilinear(
                        top, left, width, height, near_index, far_index, alpha, target,
                    );
                }
                _ => panic!(),
            },
        }
    }

    fn blit_resized_nearest(
        &self,
        top: u32,
        left: u32,
        width: u32,
        height: u32,
        level_index: usize,
        target: &mut Buffer2D,
    ) {
        for sample_y in 0..height {
            for sample_x in 0..width {
                let uv = Vec2 {
                    x: sample_x as f32 / width as f32,
                    y: 1.0 - sample_y as f32 / height as f32,
                    z: 0.0,
                };

                let sample = sample_nearest_u8(uv, self, Some(level_index));

                let (screen_x, screen_y) = (left + sample_x, top + sample_y);

                if screen_x < target.width && screen_y < target.height {
                    target.set(
                        screen_x,
                        screen_y,
                        Color::rgb(sample.0, sample.1, sample.2).to_u32(),
                    )
                }
            }
        }
    }

    fn blit_resized_bilinear(
        &self,
        top: u32,
        left: u32,
        width: u32,
        height: u32,
        level_index: usize,
        target: &mut Buffer2D,
    ) {
        for sample_y in 0..height {
            for sample_x in 0..width {
                let uv = Vec2 {
                    x: sample_x as f32 / width as f32,
                    y: 1.0 - sample_y as f32 / height as f32,
                    z: 0.0,
                };

                let sample = sample_bilinear_u8(uv, self, Some(level_index));

                let (screen_x, screen_y) = (left + sample_x, top + sample_y);

                if screen_x < target.width && screen_y < target.height {
                    target.set(
                        screen_x,
                        screen_y,
                        Color::rgb(sample.0, sample.1, sample.2).to_u32(),
                    )
                }
            }
        }
    }

    fn blit_resized_trilinear(
        &self,
        top: u32,
        left: u32,
        width: u32,
        height: u32,
        near_index: usize,
        far_index: usize,
        alpha: f32,
        target: &mut Buffer2D,
    ) {
        for sample_y in 0..height {
            for sample_x in 0..width {
                let uv = Vec2 {
                    x: sample_x as f32 / width as f32,
                    y: 1.0 - sample_y as f32 / height as f32,
                    z: 0.0,
                };

                let sample = sample_trilinear_u8(uv, self, near_index, far_index, alpha);

                let (screen_x, screen_y) = (left + sample_x, top + sample_y);

                if screen_x < target.width && screen_y < target.height {
                    target.set(
                        screen_x,
                        screen_y,
                        Color::rgb(sample.0, sample.1, sample.2).to_u32(),
                    )
                }
            }
        }
    }
}

impl TextureMap<Vec3> {
    pub fn generate_mipmaps(&mut self) -> Result<(), String> {
        self.validate_for_mipmapping()?;

        let levels = (self.width as f32).log2() + 1.0;

        // Generate each level of our mipmapped texture
        for level_index in 1..levels as usize {
            let dimension = self.width / 2_u32.pow(level_index as u32);

            let last = self.levels.last().unwrap();

            let bytes = get_half_scaled_vec3(dimension, &last.0);

            self.levels.push(TextureBuffer(Buffer2D::from_data(
                dimension, dimension, bytes,
            )));
        }

        self.has_mipmaps_generated = true;

        Ok(())
    }
}

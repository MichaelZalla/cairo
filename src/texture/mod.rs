use sdl2::image::InitFlag;
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::context::ApplicationRenderingContext;
use crate::debug_print;
use crate::graphics::pixelbuffer::PixelBuffer;

pub mod cubemap;
pub mod sample;
pub mod uv;

pub type TextureBuffer = PixelBuffer<u8>;

#[derive(Debug, Clone, Default)]
pub struct TextureMapInfo {
    pub filepath: String,
}

#[derive(Debug, Clone, Default)]
pub struct TextureMap {
    pub info: TextureMapInfo,
    pub is_loaded: bool,
    pub is_tileable: bool,
    pub is_mipmapped: bool,
    pub width: u32,
    pub height: u32,
    pub levels: Vec<TextureBuffer>,
}

impl TextureMap {
    const BYTES_PER_PIXEL: usize = 3;

    pub fn new(filepath: &str) -> Self {
        TextureMap {
            info: TextureMapInfo {
                filepath: filepath.to_string(),
            },
            is_loaded: false,
            is_tileable: false,
            is_mipmapped: false,
            width: 0,
            height: 0,
            levels: vec![],
        }
    }

    pub fn load(&mut self, rendering_context: &ApplicationRenderingContext) -> Result<(), String> {
        // Load the map's native-sized pixel data on-demand.

        let mut canvas = rendering_context.canvas.write().unwrap();

        sdl2::image::init(InitFlag::JPG).unwrap();

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

                let pixels = texture_canvas
                    .read_pixels(None, PixelFormatEnum::RGB24)
                    .unwrap();

                let mut original_size_bytes: Vec<u8> = vec![];

                original_size_bytes.resize(pixels.len(), 0);

                original_size_bytes.copy_from_slice(pixels.as_slice());

                let buffer = PixelBuffer::from_data(self.width, self.height, original_size_bytes);

                self.levels.push(buffer);
            })
            .unwrap();

        self.is_loaded = true;

        if self.is_mipmapped && self.levels.len() == 0 {
            self.make_mipmaps()?
        }

        Ok(())
    }

    pub fn enable_mipmapping(&mut self) -> Result<(), String> {
        if self.is_mipmapped {
            debug_print!("Called Texture::enable_mipmapping() on a Texture that already has mipmapping enabled!");
            return Ok(());
        }

        // Validate that this texture is suitable for mipmapping.
        let levels = (self.width as f32).log2() + 1.0;

        if self.width != self.height || levels.fract() != 0.0 {
            return Err(String::from("Called TextureMap::make_mipmaps() on a texture whose dimensions do not support mipmapping."));
        }

        self.is_mipmapped = true;

        if !self.is_loaded {
            // If the texture isn't yet loaded, return.

            return Ok(());
        }

        // Otherwise, generate the data for each mipmap level.
        self.make_mipmaps()
    }

    fn make_mipmaps(&mut self) -> Result<(), String> {
        let levels = (self.width as f32).log2() + 1.0;

        // Generate each level of our mipmapped texture
        for level_index in 1..levels as usize {
            let dimension = self.width as u32 / (2 as u32).pow(level_index as u32);

            let bytes = get_half_scaled(dimension, &self.levels.last().unwrap().data);

            self.levels
                .push(PixelBuffer::from_data(dimension, dimension, bytes));
        }

        return Ok(());
    }

    pub fn map<T>(&mut self, mut callback: T) -> Result<(), String>
    where
        T: FnMut(u8, u8, u8) -> (u8, u8, u8),
    {
        if self.is_loaded == false {
            return Err("Called TextureMap::map() on an unloaded texture!".to_string());
        }

        let original_size_buffer = &mut self.levels[0];

        for i in 0..(self.width * self.height) as usize {
            let r = original_size_buffer.data[i * 3];
            let g = original_size_buffer.data[i * 3 + 1];
            let b = original_size_buffer.data[i * 3 + 2];

            let (r_new, g_new, b_new) = callback(r, g, b);

            original_size_buffer.data[i * 3] = r_new;
            original_size_buffer.data[i * 3 + 1] = g_new;
            original_size_buffer.data[i * 3 + 2] = b_new;
        }

        Ok(())
    }
}

fn get_half_scaled(half_scaled_dimension: u32, pixel_data: &Vec<u8>) -> Vec<u8> {
    let mut result: Vec<u8> = vec![];

    let full_scale_stride = half_scaled_dimension as usize * 2 * TextureMap::BYTES_PER_PIXEL;

    let half_scale_stride = half_scaled_dimension as usize * TextureMap::BYTES_PER_PIXEL;
    let half_scaled_pixel_count = half_scaled_dimension as usize * half_scaled_dimension as usize;

    result.resize(half_scaled_pixel_count * TextureMap::BYTES_PER_PIXEL, 255);

    for small_y in 0..half_scaled_dimension as usize {
        for small_x in 0..half_scaled_dimension as usize {
            let big_y = small_y * 2;
            let big_x = small_x * 2;

            let mut r: u32 = 0;
            let mut g: u32 = 0;
            let mut b: u32 = 0;

            let top_left = big_y * full_scale_stride + big_x * TextureMap::BYTES_PER_PIXEL;
            let top_right = top_left + TextureMap::BYTES_PER_PIXEL;
            let bottom_left = top_left + full_scale_stride;
            let bottom_right = bottom_left + TextureMap::BYTES_PER_PIXEL;

            for index in [top_left, top_right, bottom_left, bottom_right].iter() {
                r += pixel_data[*index] as u32;
                g += pixel_data[*index + 1] as u32;
                b += pixel_data[*index + 2] as u32;
            }

            let half_scaled_index =
                small_y * half_scale_stride + small_x * TextureMap::BYTES_PER_PIXEL;

            let r_u8 = (r as f32 / 4.0) as u8;
            let g_u8 = (g as f32 / 4.0) as u8;
            let b_u8 = (b as f32 / 4.0) as u8;

            result[half_scaled_index] = r_u8;
            result[half_scaled_index + 1] = g_u8;
            result[half_scaled_index + 2] = b_u8;
        }
    }

    return result;
}

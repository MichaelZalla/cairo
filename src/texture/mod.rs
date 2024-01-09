use sdl2::image::InitFlag;
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::context::ApplicationRenderingContext;

pub mod sample;
pub mod uv;

#[derive(Debug, Clone, Default)]
pub struct TextureMapInfo {
    pub filepath: String,
}

#[derive(Debug, Clone, Default)]
pub struct TextureMap {
    pub info: TextureMapInfo,
    pub is_loaded: bool,
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u8>,
}

impl TextureMap {
    const BYTES_PER_PIXEL: usize = 3;

    pub fn new(filepath: &str) -> Self {
        TextureMap {
            info: TextureMapInfo {
                filepath: filepath.to_string(),
            },
            is_loaded: false,
            width: 0,
            height: 0,
            pixel_data: vec![],
        }
    }

    pub fn load(&mut self, rendering_context: &ApplicationRenderingContext) -> Result<(), String> {
        // Load the map's pixel data on-demand

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

                self.pixel_data.resize(pixels.len(), 0);
                self.pixel_data.copy_from_slice(pixels.as_slice());
            })
            .unwrap();

        self.is_loaded = true;

        Ok(())
    }

    pub fn map<T>(&mut self, mut callback: T) -> Result<(), String>
    where
        T: FnMut(u8, u8, u8) -> (u8, u8, u8),
    {
        if self.is_loaded == false {
            return Err("Called TextureMap::map() on an unloaded texture!".to_string());
        }

        for i in 0..(self.width * self.height) as usize {
            let r = self.pixel_data[i * 3];
            let g = self.pixel_data[i * 3 + 1];
            let b = self.pixel_data[i * 3 + 2];

            let (r_new, g_new, b_new) = callback(r, g, b);

            self.pixel_data[i * 3] = r_new;
            self.pixel_data[i * 3 + 1] = g_new;
            self.pixel_data[i * 3 + 2] = b_new;
        }

        Ok(())
    }
}

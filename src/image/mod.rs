use sdl2::image::InitFlag;
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::context::ApplicationRenderingContext;
use crate::vec::vec2::Vec2;

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
                    .read_pixels(None, PixelFormatEnum::RGBA32)
                    .unwrap();

                self.pixel_data.resize(pixels.len(), 0);
                self.pixel_data.copy_from_slice(pixels.as_slice());
            })
            .unwrap();

        Ok(())
    }
}

pub fn get_texture_map_from_image_path(
    filepath: String,
    rendering_context: &ApplicationRenderingContext,
) -> Result<TextureMap, String> {
    let mut map = TextureMap {
        info: TextureMapInfo { filepath },
        is_loaded: true,
        width: 0,
        height: 0,
        pixel_data: vec![],
    };

    match map.load(rendering_context) {
        Ok(()) => return Ok(map),
        Err(msg) => return Err(msg),
    }
}

pub fn sample_from_uv(uv: Vec2, map: &TextureMap) -> (u8, u8, u8, u8) {
    if map.is_loaded == false {
        panic!(
            "Called sample_from_uv() with an unloaded texture map: {}",
            map.info.filepath
        );
    }

    let uv_x_safe = if uv.x < 0.0 || uv.x >= 1.0 {
        uv.x.rem_euclid(1.0)
    } else {
        uv.x
    };

    let uv_y_safe = if uv.y < 0.0 || uv.y >= 1.0 {
        uv.y.rem_euclid(1.0)
    } else {
        uv.y
    };

    assert!(map.pixel_data.len() == (map.width * map.height * 4) as usize);

    let texel_x = (((1.0 - uv_x_safe) * (map.width - 1) as f32).floor()) as u32;
    let texel_y = (((1.0 - uv_y_safe) * (map.height - 1) as f32).floor()) as u32;

    let texel_color_index = 4 * (texel_y * map.width + texel_x) as usize;

    let pixels = &map.pixel_data;

    assert!(texel_color_index < pixels.len());

    let r: u8 = pixels[texel_color_index];
    let g: u8 = pixels[texel_color_index + 1];
    let b: u8 = pixels[texel_color_index + 2];
    let a: u8 = pixels[texel_color_index + 3];

    return (r, g, b, a);
}

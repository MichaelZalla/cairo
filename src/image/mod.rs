use sdl2::image::InitFlag;
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::context::ApplicationRenderingContext;

#[derive(Debug, Clone, Default)]
pub struct TextureMap {
    pub filepath: String,
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u8>,
}

pub fn get_texture_map_from_image_path(
    filepath: String,
    rendering_context: &ApplicationRenderingContext,
) -> TextureMap {
    sdl2::image::init(InitFlag::JPG).unwrap();

    let mut pixel_data: Vec<u8> = vec![];

    let mut canvas = rendering_context.canvas.write().unwrap();

    let texture_creator = canvas.texture_creator();

    let static_texture = texture_creator.load_texture(filepath.clone()).unwrap();

    let texture_attrs = static_texture.query();
    let width = texture_attrs.width;
    let height = texture_attrs.height;

    let mut target_texture = texture_creator
        .create_texture(
            PixelFormatEnum::RGBA32,
            TextureAccess::Target,
            width,
            height,
        )
        .unwrap();

    canvas
        .with_texture_canvas(&mut target_texture, |texture_canvas| {
            texture_canvas.copy(&static_texture, None, None).unwrap();

            let pixels = texture_canvas
                .read_pixels(None, PixelFormatEnum::RGBA32)
                .unwrap();

            pixel_data.resize(pixels.len(), 0);
            pixel_data.copy_from_slice(pixels.as_slice());
        })
        .unwrap();

    return TextureMap {
        filepath,
        width,
        height,
        pixel_data,
    };
}

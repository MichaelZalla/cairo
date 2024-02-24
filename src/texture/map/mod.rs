use sdl2::image::InitFlag;
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::buffer::Buffer2D;
use crate::context::ApplicationRenderingContext;

use crate::debug_print;

use super::get_half_scaled;

pub type TextureBuffer = Buffer2D<u8>;

#[derive(Default, Debug, Copy, Clone)]
pub enum TextureMapStorageFormat {
    #[default]
    RGBA32,
    RGB24,
    Index8,
}

impl TextureMapStorageFormat {
    pub fn get_bytes_per_pixel(&self) -> usize {
        match self {
            TextureMapStorageFormat::RGBA32 => 4,
            TextureMapStorageFormat::RGB24 => 3,
            TextureMapStorageFormat::Index8 => 1,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TextureMapInfo {
    pub filepath: String,
    pub storage_format: TextureMapStorageFormat,
    pub target_channel: usize,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum TextureMapWrapping {
    #[default]
    Repeat,
    ClampToEdge,
    ClampToBorder,
}

#[derive(Default, Debug, Clone)]
pub struct TextureMapOptions {
    pub wrapping: TextureMapWrapping,
    pub border_color: Option<(u8, u8, u8)>,
}

#[derive(Default, Debug, Clone)]
pub struct TextureMap {
    pub info: TextureMapInfo,
    pub is_loaded: bool,
    pub is_mipmapped: bool,
    pub width: u32,
    pub height: u32,
    pub levels: Vec<TextureBuffer>,
    pub options: TextureMapOptions,
}

impl TextureMap {
    pub fn new(filepath: &str, storage_format: TextureMapStorageFormat) -> Self {
        Self {
            info: TextureMapInfo {
                filepath: filepath.to_string(),
                storage_format,
                target_channel: 0,
            },
            is_loaded: false,
            is_mipmapped: false,
            width: 0,
            height: 0,
            levels: vec![],
            options: Default::default(),
        }
    }

    pub fn from_buffer(width: u32, height: u32, buffer: &Buffer2D<u8>) -> Self {
        let bytes_per_pixel = buffer.data.len() as u32 / width / height;

        return Self {
            info: TextureMapInfo {
                filepath: "Buffer".to_string(),
                storage_format: if bytes_per_pixel == 4 {
                    TextureMapStorageFormat::RGBA32
                } else if bytes_per_pixel == 3 {
                    TextureMapStorageFormat::RGB24
                } else if bytes_per_pixel == 1 {
                    TextureMapStorageFormat::Index8
                } else {
                    panic!(
                        "Invalid buffer data length {} for buffer size {}x{}!",
                        buffer.data.len(),
                        width,
                        height
                    )
                },
                target_channel: 0,
            },
            is_loaded: true,
            is_mipmapped: false,
            width,
            height,
            levels: vec![buffer.clone()],
            options: Default::default(),
        };
    }

    pub fn from_alpha_channel(filepath: &str) -> Self {
        let mut map = Self::new(filepath, TextureMapStorageFormat::Index8);

        map.info.target_channel = PixelFormatEnum::RGBA32.byte_size_per_pixel() - 1;

        map
    }

    pub fn load(&mut self, rendering_context: &ApplicationRenderingContext) -> Result<(), String> {
        // Load the map's native-sized pixel data on-demand.

        let mut canvas = rendering_context.canvas.borrow_mut();

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

                let sdl_read_pixel_format = match self.info.storage_format {
                    TextureMapStorageFormat::RGBA32 => {
                        debug_assert!(
                            TextureMapStorageFormat::RGBA32.get_bytes_per_pixel()
                                == PixelFormatEnum::RGBA32.byte_size_per_pixel()
                        );
                        PixelFormatEnum::RGBA32
                    }
                    TextureMapStorageFormat::RGB24 => {
                        debug_assert!(
                            TextureMapStorageFormat::RGB24.get_bytes_per_pixel()
                                == PixelFormatEnum::RGB24.byte_size_per_pixel()
                        );
                        PixelFormatEnum::RGB24
                    }
                    // Err: "Indexed pixel formats not supported"
                    TextureMapStorageFormat::Index8 => PixelFormatEnum::RGBA32,
                };

                let bytes_per_src_pixel = sdl_read_pixel_format.byte_size_per_pixel();
                let bytes_per_dest_pixel = self.info.storage_format.get_bytes_per_pixel();

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
                    TextureMapStorageFormat::Index8 => {
                        original_size_bytes.resize(pixels_bytes / bytes_per_src_pixel, 0);

                        if self.info.target_channel >= bytes_per_src_pixel {
                            panic!("Invalid channel offset ({}) for texture map with Index8 storage format!", self.info.target_channel);
                        }

                        for i in 0..pixels_bytes / bytes_per_src_pixel {
                            original_size_bytes[i] = pixels[i * bytes_per_src_pixel + self.info.target_channel];
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

                self.levels.push(buffer);
            })
            .unwrap();

        self.is_loaded = true;

        if self.is_mipmapped && self.levels.len() == 0 {
            self.make_mipmaps()?
        }

        Ok(())
    }

    pub fn get_bytes_per_pixel(&self) -> usize {
        self.info.storage_format.get_bytes_per_pixel()
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

            let bytes = get_half_scaled(dimension, &self.levels.last().unwrap());

            self.levels
                .push(Buffer2D::from_data(dimension, dimension, bytes));
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

        let bytes_per_pixel = self.get_bytes_per_pixel();

        let original_size_buffer = &mut self.levels[0];

        for i in 0..(self.width * self.height) as usize {
            let r = original_size_buffer.data[i * bytes_per_pixel];
            let g;
            let b;

            match self.info.storage_format {
                TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                    g = original_size_buffer.data[i * bytes_per_pixel + 1];
                    b = original_size_buffer.data[i * bytes_per_pixel + 2];
                }
                TextureMapStorageFormat::Index8 => {
                    g = r;
                    b = r;
                }
            }

            let (r_new, g_new, b_new) = callback(r, g, b);

            original_size_buffer.data[i * bytes_per_pixel] = r_new;

            match self.info.storage_format {
                TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
                    original_size_buffer.data[i * bytes_per_pixel + 1] = g_new;
                    original_size_buffer.data[i * bytes_per_pixel + 2] = b_new;
                }
                TextureMapStorageFormat::Index8 => (),
            }
        }

        Ok(())
    }
}

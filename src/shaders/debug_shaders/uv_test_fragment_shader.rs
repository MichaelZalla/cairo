use std::sync::RwLock;

use crate::{
    color::Color,
    device::{GameControllerState, KeyboardState, MouseState},
    shader::{
        fragment::{FragmentShader, FragmentShaderOptions},
        ShaderContext,
    },
    texture::{
        sample::{sample_bilinear, sample_nearest},
        TextureMap,
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct UvTestFragmentShader<'a> {
    options: FragmentShaderOptions,
    context: &'a RwLock<ShaderContext>,
    texture_map: Option<TextureMap>,
}

impl<'a> UvTestFragmentShader<'a> {
    pub fn from_texture_map(
        context: &'a RwLock<ShaderContext>,
        texture_map: TextureMap,
        options: Option<FragmentShaderOptions>,
    ) -> Self {
        let mut shader = UvTestFragmentShader::new(context, options);

        shader.texture_map = Some(texture_map);

        shader
    }
}

impl<'a> FragmentShader<'a> for UvTestFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<FragmentShaderOptions>) -> Self {
        match options {
            Some(options) => Self {
                context,
                options,
                texture_map: None,
            },
            None => Self {
                context,
                options: Default::default(),
                texture_map: None,
            },
        }
    }

    fn update(
        &mut self,
        _keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        _game_controller_state: &GameControllerState,
    ) {
        // Do nothing
    }

    fn call(&self, out: &DefaultVertexOut) -> Color {
        // Emit an RGB representation of this fragment's interpolated UV.

        let r: u8;
        let g: u8;
        let b: u8;

        match &self.texture_map {
            Some(texture) => {
                (r, g, b) = if self.options.bilinear_active {
                    sample_bilinear(out.uv, texture, None)
                } else {
                    sample_nearest(out.uv, texture, None)
                };
            }
            None => {
                r = (out.uv.x * 255.0) as u8;
                g = (out.uv.y * 255.0) as u8;
                b = (out.uv.z * 255.0) as u8;
            }
        }

        return Color::rgb(r, g, b);
    }
}

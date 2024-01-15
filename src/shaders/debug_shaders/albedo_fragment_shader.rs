use std::sync::RwLock;

use crate::{
    color::Color,
    device::{GameControllerState, KeyboardState, MouseState},
    shader::{
        fragment::{FragmentShader, FragmentShaderOptions},
        ShaderContext,
    },
    texture::sample::{sample_bilinear, sample_nearest},
    vec::vec3::Vec3,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct AlbedoFragmentShader<'a> {
    options: FragmentShaderOptions,
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for AlbedoFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<FragmentShaderOptions>) -> Self {
        match options {
            Some(options) => Self { context, options },
            None => Self {
                context,
                options: Default::default(),
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
        let context: std::sync::RwLockReadGuard<'_, ShaderContext> = self.context.read().unwrap();

        // Emit only the diffuse color for this fragment.

        let mut color: Vec3 = out.c;

        match context.active_material {
            Some(mat_raw_mut) => unsafe {
                match (
                    self.options.diffuse_mapping_active,
                    &(*mat_raw_mut).diffuse_map,
                ) {
                    (true, Some(texture)) => {
                        let (r, g, b) = if self.options.bilinear_active {
                            sample_bilinear(out.uv, texture, None)
                        } else {
                            sample_nearest(out.uv, texture, None)
                        };

                        color = Color::rgb(r, g, b).to_vec3();
                    }
                    _ => {
                        color = (*mat_raw_mut).diffuse_color;
                    }
                }
            },
            None => {}
        }

        return Color {
            r: color.x as u8,
            g: color.y as u8,
            b: color.z as u8,
            a: 255 as u8,
        };
    }
}

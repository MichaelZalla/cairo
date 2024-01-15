use std::sync::RwLock;

use crate::{
    color::Color,
    device::{GameControllerState, KeyboardState, MouseState},
    shader::{
        fragment::{FragmentShader, FragmentShaderOptions},
        ShaderContext,
    },
    texture::sample::sample_nearest,
    vec::vec4::Vec4,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct NormalFragmentShader<'a> {
    options: FragmentShaderOptions,
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for NormalFragmentShader<'a> {
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

        // Emit only the world-space normal (RBG space) for this fragment.

        let surface_normal = out.n.as_normal();

        // let surface_normal_vec3 = Vec3 {
        //     x: surface_normal.x,
        //     y: surface_normal.y,
        //     z: surface_normal.z,
        // };

        match (self.options.normal_mapping_active, context.active_material) {
            (true, Some(mat_raw_mut)) => {
                unsafe {
                    match &(*mat_raw_mut).normal_map {
                        Some(texture) => {
                            let (r, g, b) = sample_nearest(out.uv, texture, None);

                            let _map_normal = Vec4 {
                                x: (r as f32 / 255.0) * 2.0 - 1.0,
                                y: (g as f32 / 255.0) * 2.0 - 1.0,
                                z: (b as f32 / 255.0) * 2.0 - 1.0,
                                w: 1.0,
                            };

                            // @TODO Perturb the surface normal using the local
                            // tangent-space information read from `map`
                            //
                            // surface_normal = (surface_normal * out.TBN).as_normal();
                        }
                        None => (),
                    }
                }
            }
            _ => (),
        }

        return Color {
            r: (surface_normal.x * 255.0) as u8,
            g: (surface_normal.y * 255.0) as u8,
            b: (surface_normal.z * 255.0) as u8,
            a: 255 as u8,
        };
    }
}

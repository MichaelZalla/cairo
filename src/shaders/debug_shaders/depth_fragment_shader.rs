use std::sync::RwLock;

use crate::{
    color::Color,
    device::{GameControllerState, KeyboardState, MouseState},
    shader::{
        fragment::{FragmentShader, FragmentShaderOptions},
        ShaderContext,
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct DepthFragmentShader<'a> {
    options: FragmentShaderOptions,
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for DepthFragmentShader<'a> {
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
        // Emit only the linear depth value (in RGB space) for this fragment.

        let non_linear_depth: f32 = out.depth;

        //               nlz = (1/z - 1/n) / (1/f - 1/n)
        // nlz * (1/f - 1/n) = 1/z - 1/n
        //               1/z = nlz * (1/f - 1/n) + 1/n
        //                 z = 1 / (nlz * (1/f - 1/n) + 1/n)

        let linear_depth = 1.0 / (non_linear_depth * (1.0 / 10.0 - 1.0 / 0.3) + 1.0 / 0.3);

        // @NOTE May need to account for diplacement map in the future.
        return Color {
            r: ((1.0 - linear_depth) * 255.0) as u8,
            g: ((1.0 - linear_depth) * 255.0) as u8,
            b: ((1.0 - linear_depth) * 255.0) as u8,
            a: 255 as u8,
        };
    }
}

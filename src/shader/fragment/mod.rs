use crate::{color::Color, scene::resources::SceneResources};

use super::{context::ShaderContext, geometry::sample::GeometrySample};

pub type FragmentShaderFn = fn(&ShaderContext, &SceneResources, &GeometrySample) -> Color;

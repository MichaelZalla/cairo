use crate::color::Color;

use super::{context::ShaderContext, geometry::sample::GeometrySample};

pub type FragmentShaderFn = fn(&ShaderContext, &GeometrySample) -> Color;

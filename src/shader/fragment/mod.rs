use std::sync::RwLockReadGuard;

use crate::color::Color;

use super::{context::ShaderContext, geometry::sample::GeometrySample};

pub type FragmentShaderFn = fn(&RwLockReadGuard<ShaderContext>, &GeometrySample) -> Color;

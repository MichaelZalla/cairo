use std::sync::RwLockReadGuard;

use crate::color::Color;

use super::{geometry::sample::GeometrySample, ShaderContext};

pub type FragmentShaderFn = fn(&RwLockReadGuard<ShaderContext>, &GeometrySample) -> Color;

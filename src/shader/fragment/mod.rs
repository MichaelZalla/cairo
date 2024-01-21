use std::sync::RwLock;

use crate::color::Color;

use super::{geometry::sample::GeometrySample, ShaderContext};

pub trait FragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self;

    fn call(&self, sample: &GeometrySample) -> Color;
}

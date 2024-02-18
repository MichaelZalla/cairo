use crate::vertex::default_vertex_out::DefaultVertexOut;

use self::{options::GeometryShaderOptions, sample::GeometrySample};

pub mod options;
pub mod sample;

use std::sync::RwLockReadGuard;

use super::context::ShaderContext;

pub type GeometryShaderFn = fn(
    &RwLockReadGuard<ShaderContext>,
    &GeometryShaderOptions,
    &DefaultVertexOut,
) -> Option<GeometrySample>;

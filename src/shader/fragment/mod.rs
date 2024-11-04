use crate::{scene::resources::SceneResources, vec::vec3::Vec3};

use super::{context::ShaderContext, geometry::sample::GeometrySample};

pub type FragmentShaderFn = fn(&ShaderContext, &SceneResources, &GeometrySample) -> Vec3;

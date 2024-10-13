use crate::render::culling::FaceCullingStrategy;

#[derive(Default, Debug, Copy, Clone)]
pub struct RasterizerOptions {
    pub face_culling_strategy: FaceCullingStrategy,
}

#[derive(Debug, Copy, Clone)]
pub struct VertexShaderOptions {
    pub uv_mapping_active: bool,
    pub normal_mapping_active: bool,
}

impl Default for VertexShaderOptions {
    fn default() -> Self {
        Self {
            uv_mapping_active: true,
            normal_mapping_active: false,
        }
    }
}

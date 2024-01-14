#[derive(Copy, Clone, Default)]
pub struct PipelineOptions {
    pub should_render_wireframe: bool,
    pub should_render_shader: bool,
    pub should_render_normals: bool,
    pub should_cull_backfaces: bool,
}

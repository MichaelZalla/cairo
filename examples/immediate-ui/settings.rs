use cairo::{
    app::window::AppWindowingMode,
    render::options::{shader::RenderShaderOptions, RenderOptions},
    software_renderer::zbuffer::DepthTestMethod,
};

#[derive(Default, Debug, Clone)]
pub(crate) struct Settings {
    pub windowing_mode: AppWindowingMode,
    pub resolution: usize,
    pub vsync: bool,
    pub hdr: bool,
    pub render_options: RenderOptions,
    pub shader_options: RenderShaderOptions,
    pub depth_test_method: DepthTestMethod,
    pub fragment_shader: usize,
    pub tone_mapping: usize,
}

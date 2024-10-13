use cairo::{
    app::window::AppWindowingMode, render::options::shader::RenderShaderOptions,
    software_renderer::zbuffer::DepthTestMethod,
};

#[derive(Default, Debug, Clone)]
pub(crate) struct Settings {
    pub windowing_mode: AppWindowingMode,
    pub resolution: usize,
    pub vsync: bool,
    pub hdr: bool,
    pub bloom: bool,
    pub shader_options: RenderShaderOptions,
    pub depth_test_method: DepthTestMethod,
}

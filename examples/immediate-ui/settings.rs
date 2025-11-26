use cairo::{
    app::window::AppWindowingMode,
    render::options::{RenderOptions, shader::RenderShaderOptions},
    software_renderer::zbuffer::DepthTestMethod,
};

#[derive(Default, Debug, Clone)]
pub struct ActiveEffects {
    pub outline: bool,
    pub invert: bool,
    pub grayscale: bool,
    pub sharpen_kernel: bool,
    pub blur_kernel: bool,
    pub edge_detection_kernel: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Settings {
    pub windowing_mode: AppWindowingMode,
    pub resolution: usize,
    pub vsync: bool,
    pub hdr: bool,
    pub brightness: f32,
    pub gamma: f32,
    pub render_options: RenderOptions,
    pub shader_options: RenderShaderOptions,
    pub depth_test_method: DepthTestMethod,
    pub fragment_shader: usize,
    pub effects: ActiveEffects,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            windowing_mode: Default::default(),
            resolution: Default::default(),
            vsync: true,
            hdr: true,
            brightness: 0.8,
            gamma: 2.2,
            render_options: Default::default(),
            shader_options: Default::default(),
            depth_test_method: Default::default(),
            fragment_shader: Default::default(),
            effects: Default::default(),
        }
    }
}

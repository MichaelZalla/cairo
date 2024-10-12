use cairo::app::window::AppWindowingMode;

#[derive(Default, Debug, Clone)]
pub(crate) struct Settings {
    pub windowing_mode: AppWindowingMode,
    pub resolution: usize,
    pub vsync: bool,
    pub hdr: bool,
    pub bloom: bool,
}

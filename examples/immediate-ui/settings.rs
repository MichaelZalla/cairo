#[derive(Default, Debug, Clone)]
pub(crate) struct Settings {
    pub clicked_count: usize,
    pub vsync: bool,
    pub hdr: bool,
    pub bloom: bool,
}

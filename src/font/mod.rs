pub mod cache;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FontInfo {
    pub filepath: String,
    pub point_size: u16,
}

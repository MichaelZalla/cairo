use core::fmt;

pub mod cache;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FontInfo {
    pub filepath: String,
    pub point_size: u16,
}

impl fmt::Display for FontInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FontInfo('{}', {})", self.filepath, self.point_size)
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod load;
pub mod rgbe;

pub static HDR_FILE_PRELUDE: &str = "#?RADIANCE";

pub static HDR_CHANNELS_PER_SAMPLE: usize = 4;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum HdrRadianceFormat {
    // 32-bit_rle_rgbe
    #[default]
    RleRgbe32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HdrSource {
    pub filename: String,
    pub radiance_format: HdrRadianceFormat,
    pub width: usize,
    pub height: usize,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Hdr {
    pub source: HdrSource,
    pub headers: HashMap<String, String>,
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::vec::vec3::Vec3;

use self::rgbe::Rgbe;

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

impl Hdr {
    pub fn to_vec3(&self) -> Vec<Vec3> {
        let mut vecs = Vec::<Vec3>::with_capacity(self.bytes.len() / HDR_CHANNELS_PER_SAMPLE);

        for chunk in self.bytes.chunks(4) {
            vecs.push(
                Rgbe {
                    r: chunk[0],
                    g: chunk[1],
                    b: chunk[2],
                    e: chunk[3],
                }
                .to_vec3(),
            );
        }

        vecs
    }
}

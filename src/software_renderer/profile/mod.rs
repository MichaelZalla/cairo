use core::fmt;

use crate::stats::FromIndex;

pub enum SoftwareRendererCycleCounter {
    BeginAndEndFrame,
    Count,
}

impl fmt::Display for SoftwareRendererCycleCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::BeginAndEndFrame => "BeginAndEndFrame",
                _ => panic!(),
            }
        )
    }
}

impl FromIndex for SoftwareRendererCycleCounter {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::BeginAndEndFrame,
            _ => panic!(),
        }
    }
}

use core::fmt;

use crate::stats::FromIndex;

pub enum AppCycleCounter {
    Run,
    UpdateCallback,
    RenderCallback,
    RenderAndPresent,
    CopyAndPresent,
    Count,
}

impl fmt::Display for AppCycleCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Run => "Run",
                Self::UpdateCallback => "UpdateCallback",
                Self::RenderCallback => "RenderCallback",
                Self::RenderAndPresent => "RenderAndPresent",
                Self::CopyAndPresent => "CopyAndPresent",
                _ => panic!(),
            }
        )
    }
}

impl FromIndex for AppCycleCounter {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Run,
            1 => Self::UpdateCallback,
            2 => Self::RenderCallback,
            3 => Self::RenderAndPresent,
            4 => Self::CopyAndPresent,
            _ => panic!(),
        }
    }
}

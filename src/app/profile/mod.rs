use core::fmt;

pub enum AppCycleCounter {
    Run,
    UpdateCallback,
    RenderCallback,
    RenderAndPresent,
    Count,
}

impl fmt::Display for AppCycleCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppCycleCounter::Run => "Run",
                AppCycleCounter::UpdateCallback => "UpdateCallback",
                AppCycleCounter::RenderCallback => "RenderCallback",
                AppCycleCounter::RenderAndPresent => "RenderAndPresent",
                _ => panic!(),
            }
        )
    }
}

impl AppCycleCounter {
    pub fn from(index: usize) -> Self {
        match index {
            0 => Self::Run,
            1 => Self::UpdateCallback,
            2 => Self::RenderCallback,
            3 => Self::RenderAndPresent,
            _ => panic!(),
        }
    }
}

use std::fmt;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AppWindowingMode {
    #[default]
    Windowed = 0,
    FullScreen = 1,
    FullScreenWindowed = 2,
}

impl fmt::Display for AppWindowingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppWindowingMode::Windowed => write!(f, "Windowed"),
            AppWindowingMode::FullScreen => write!(f, "Fullscreen"),
            AppWindowingMode::FullScreenWindowed => write!(f, "Fullscreen windowed"),
        }
    }
}

pub static APP_WINDOWING_MODES: [AppWindowingMode; 3] = [
    AppWindowingMode::Windowed,
    AppWindowingMode::FullScreen,
    AppWindowingMode::FullScreenWindowed,
];

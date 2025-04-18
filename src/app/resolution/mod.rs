use std::{
    fmt,
    ops::{Mul, MulAssign},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        RESOLUTION_960_BY_540
    }
}

impl MulAssign<f32> for Resolution {
    fn mul_assign(&mut self, scale: f32) {
        self.width = (self.width as f32 * scale) as u32;
        self.height = (self.height as f32 * scale) as u32;
    }
}

impl Mul<f32> for Resolution {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        let mut cloned = self;
        cloned *= scale;
        cloned
    }
}

impl Resolution {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            width: size.0,
            height: size.1,
        }
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resolution ({}x{})", self.width, self.height)
    }
}

pub static RESOLUTION_320_BY_180: Resolution = Resolution {
    width: 320,
    height: 180,
};

pub static RESOLUTION_480_BY_270: Resolution = Resolution {
    width: 480,
    height: 270,
};

pub static RESOLUTION_640_BY_320: Resolution = Resolution {
    width: 640,
    height: 320,
};

pub static RESOLUTION_640_BY_480: Resolution = Resolution {
    width: 640,
    height: 480,
};

pub static RESOLUTION_800_BY_450: Resolution = Resolution {
    width: 800,
    height: 450,
};

pub static RESOLUTION_960_BY_540: Resolution = Resolution {
    width: 960,
    height: 540,
};

pub static RESOLUTION_1024_BY_576: Resolution = Resolution {
    width: 1024,
    height: 576,
};

pub static RESOLUTION_1200_BY_675: Resolution = Resolution {
    width: 1200,
    height: 675,
};

pub static RESOLUTION_1280_BY_720: Resolution = Resolution {
    width: 1280,
    height: 720,
};

pub static RESOLUTION_1366_BY_768: Resolution = Resolution {
    width: 1366,
    height: 768,
};

pub static RESOLUTION_1600_BY_900: Resolution = Resolution {
    width: 1600,
    height: 900,
};

pub static RESOLUTION_1920_BY_1080: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

pub static RESOLUTION_2560_BY_1440: Resolution = Resolution {
    width: 2560,
    height: 1440,
};

pub static RESOLUTIONS_16X9: [Resolution; 11] = [
    RESOLUTION_320_BY_180,
    RESOLUTION_640_BY_320,
    RESOLUTION_800_BY_450,
    RESOLUTION_960_BY_540,
    RESOLUTION_1024_BY_576,
    RESOLUTION_1200_BY_675,
    RESOLUTION_1280_BY_720,
    RESOLUTION_1366_BY_768,
    RESOLUTION_1600_BY_900,
    RESOLUTION_1920_BY_1080,
    RESOLUTION_2560_BY_1440,
];

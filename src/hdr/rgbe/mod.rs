use crate::vec::vec3::Vec3;

pub struct Rgbe {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub e: u8,
}

impl Rgbe {
    pub fn to_vec3(&self) -> Vec3 {
        if self.e > 0 {
            let f = 1.0 * 2f32.powi((self.e as isize - (128 + 8)) as i32);

            Vec3 {
                x: self.r as f32 * f,
                y: self.g as f32 * f,
                z: self.b as f32 * f,
            }
        } else {
            Default::default()
        }
    }
}

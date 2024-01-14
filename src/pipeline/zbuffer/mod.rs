// @TODO Read near and far values from active camera.
static NEAR: f32 = 0.5;
static NEAR_RECIPROCAL: f32 = 1.0 / NEAR;
static FAR: f32 = 10000.0;
static FAR_RECIPROCAL: f32 = 1.0 / FAR;

pub static MAX_DEPTH: f32 = 1.0;

pub struct ZBuffer(pub Vec<f32>, usize);

impl ZBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let len = width as usize * height as usize;

        let mut buffer: Vec<f32> = Vec::with_capacity(len);

        for _ in 0..len {
            buffer.push(MAX_DEPTH);
        }

        Self(buffer, width as usize)
    }

    pub fn clear(&mut self) {
        for i in 0..self.0.len() {
            self.0[i] = MAX_DEPTH;
        }
    }

    pub fn test(&mut self, x: u32, y: u32, z: f32) -> Option<(usize, f32)> {
        let index = (y * self.1 as u32 + x) as usize;

        if index >= self.0.len() {
            panic!(
                "Call to ZBuffer.test() with invalid coordinate ({},{})!",
                x, y
            );
        }

        // Non-linear depth test
        // https://youtu.be/3xGKu4T4SCU?si=v7nkYrg2sFYozfZ5&t=139

        // (1/z - 1/n) / (1/f - 1/n)
        let non_linear_z = (1.0 / z - NEAR_RECIPROCAL) / (FAR_RECIPROCAL - NEAR_RECIPROCAL);

        if non_linear_z < self.0[index] {
            Some((index, non_linear_z))
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, non_linear_z: f32) {
        self.0[index] = non_linear_z;
    }
}

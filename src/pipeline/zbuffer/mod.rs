pub static MAX_DEPTH: f32 = 1.0;

pub struct ZBuffer {
    pub values: Vec<f32>,
    stride: usize,
    projection_z_near: f32,
    projection_z_far: f32,
    projection_z_near_reciprocal: f32,
    projection_z_far_reciprocal: f32,
}

impl ZBuffer {
    pub fn new(width: u32, height: u32, projection_z_near: f32, projection_z_far: f32) -> Self {
        let len = width as usize * height as usize;

        let mut buffer: Vec<f32> = Vec::with_capacity(len);

        for _ in 0..len {
            buffer.push(MAX_DEPTH);
        }

        Self {
            values: buffer,
            stride: width as usize,
            projection_z_near,
            projection_z_near_reciprocal: 1.0 / projection_z_near,
            projection_z_far,
            projection_z_far_reciprocal: 1.0 / projection_z_far,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.values.len() {
            self.values[i] = MAX_DEPTH;
        }
    }

    pub fn test(&mut self, x: u32, y: u32, z: f32) -> Option<(usize, f32)> {
        let index = (y * self.stride as u32 + x) as usize;

        if index >= self.values.len() {
            panic!(
                "Call to ZBuffer.test() with invalid coordinate ({},{})!",
                x, y
            );
        }

        // Non-linear depth test
        // https://youtu.be/3xGKu4T4SCU?si=v7nkYrg2sFYozfZ5&t=139

        // (1/z - 1/n) / (1/f - 1/n)

        let non_linear_z = (1.0 / z - self.projection_z_near_reciprocal)
            / (self.projection_z_far_reciprocal - self.projection_z_near_reciprocal);

        if non_linear_z < self.values[index] {
            Some((index, non_linear_z))
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, non_linear_z: f32) {
        self.values[index] = non_linear_z;
    }
}

use crate::buffer::Buffer2D;

pub static MAX_DEPTH: f32 = 1.0;

pub struct ZBuffer {
    buffer: Buffer2D<f32>,
    projection_z_near: f32,
    projection_z_far: f32,
    projection_z_near_reciprocal: f32,
    projection_z_far_reciprocal: f32,
}

impl ZBuffer {
    pub fn new(width: u32, height: u32, projection_z_near: f32, projection_z_far: f32) -> Self {
        let buffer = Buffer2D::<f32>::new(width, height, Some(MAX_DEPTH));

        Self {
            buffer,
            projection_z_near,
            projection_z_near_reciprocal: 1.0 / projection_z_near,
            projection_z_far,
            projection_z_far_reciprocal: 1.0 / projection_z_far,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear(Some(MAX_DEPTH));
    }

    pub fn test(&mut self, x: u32, y: u32, z: f32) -> Option<((u32, u32), f32)> {
        // Non-linear depth test
        // https://youtu.be/3xGKu4T4SCU?si=v7nkYrg2sFYozfZ5&t=139

        // (1/z - 1/n) / (1/f - 1/n)

        let non_linear_z = (1.0 / z - self.projection_z_near_reciprocal)
            / (self.projection_z_far_reciprocal - self.projection_z_near_reciprocal);

        if non_linear_z < *self.buffer.get(x, y) {
            Some(((x, y), non_linear_z))
        } else {
            None
        }
    }

    pub fn set(&mut self, x: u32, y: u32, non_linear_z: f32) {
        self.buffer.set(x, y, non_linear_z)
    }

    pub fn iter(&mut self) -> std::slice::Iter<'_, f32> {
        self.buffer.iter()
    }
}

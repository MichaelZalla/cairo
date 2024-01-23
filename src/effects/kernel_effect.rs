use crate::{
    buffer::Buffer2D,
    color::Color,
    effect::{kernel::get_coordinates, Effect},
    vec::vec3::Vec3,
};

pub struct KernelEffect {
    total: i32,
    weights: [i32; 9],
}

impl KernelEffect {
    pub fn new(weights: [i32; 9]) -> Self {
        let total = weights.iter().fold(0, |acc, value| acc + value);

        debug_assert!(total == 1);

        Self { total, weights }
    }
}

impl Effect for KernelEffect {
    fn apply(&self, buffer: &mut Buffer2D) {
        let mut out = Buffer2D::new(buffer.width, buffer.height, None);

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let mut sum: Vec3 = Default::default();

                // Perform weighted averaging of pixel and its neighbors.
                for (index, (n_x, n_y)) in get_coordinates(x as i32, y as i32).iter().enumerate() {
                    // Perform bounds-checking.
                    if *n_x < 0
                        || *n_x > (buffer.width - 1) as i32
                        || *n_y < 0
                        || *n_y > (buffer.height - 1) as i32
                    {
                        continue;
                    }

                    let color = Color::from_u32(*buffer.get(*n_x as u32, *n_y as u32)).to_vec3();

                    sum += color * self.weights[index] as f32;
                }

                out.set(x, y, Color::from_vec3(sum / self.total as f32).to_u32());
            }
        }

        buffer.blit(0, 0, buffer.width, buffer.height, out.get_all());
    }
}

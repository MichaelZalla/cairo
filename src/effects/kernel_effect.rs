use crate::{
    buffer::{get_3x3_coordinates, Buffer2D},
    color::Color,
    effect::Effect,
    vec::vec3::Vec3,
};

pub struct KernelEffect {
    accumulation_buffer: Buffer2D,
    rounds: u8,
    total: i32,
    weights: [i32; 9],
}

impl KernelEffect {
    pub fn new(weights: [i32; 9], rounds: Option<u8>) -> Self {
        let accumulation_buffer = Default::default();

        let total = weights.iter().sum::<i32>();

        let rounds = if let Some(value) = rounds { value } else { 1 };

        Self {
            accumulation_buffer,
            total,
            rounds,
            weights,
        }
    }
}

impl Effect for KernelEffect {
    fn apply(&mut self, buffer: &mut Buffer2D) {
        self.accumulation_buffer.resize(buffer.width, buffer.height);

        let mut sum: Vec3;

        for _ in 0..self.rounds {
            for y in 0..buffer.height {
                for x in 0..buffer.width {
                    sum = Default::default();

                    // Compute a weighted average of this pixel and its 8 neighbors.
                    for (index, (n_x, n_y)) in
                        get_3x3_coordinates(x as i32, y as i32).iter().enumerate()
                    {
                        let weight = self.weights[index] as f32;

                        // Perform bounds-checking.
                        if *n_x < 0
                            || *n_x > (buffer.width - 1) as i32
                            || *n_y < 0
                            || *n_y > (buffer.height - 1) as i32
                        {
                            // @TODO How to maintain image brightness when missing a neighbor?
                            continue;
                        }

                        let sample =
                            Color::from_u32(*buffer.get(*n_x as u32, *n_y as u32)).to_vec3();

                        sum += sample * weight;
                    }

                    let averaged = Color::from_vec3(sum / self.total as f32).to_u32();

                    self.accumulation_buffer.set(x, y, averaged);
                }
            }

            buffer.copy(self.accumulation_buffer.get_all().as_slice());
        }
    }
}

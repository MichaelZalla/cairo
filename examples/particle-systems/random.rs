use std::f32::consts::PI;

use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};

use cairo::vec::vec3::Vec3;

#[derive(Default, Debug, Clone)]
pub struct RandomSampler {
    rng: ThreadRng,
}

pub trait RangeSampler {
    fn sample_range_uniform(&mut self, min: f32, max: f32) -> f32;
}

pub trait DirectionSampler {
    fn sample_direction_uniform(&mut self) -> Vec3;
}

impl RangeSampler for RandomSampler {
    // Returns a uniformly distributed random scalar in the range [min...max].
    fn sample_range_uniform(&mut self, min: f32, max: f32) -> f32 {
        let sampler = Uniform::new_inclusive(min, max);

        sampler.sample(&mut self.rng)
    }
}

impl DirectionSampler for RandomSampler {
    // Returns a uniformly random normal on the unit circle.
    fn sample_direction_uniform(&mut self) -> Vec3 {
        let azimuth = self.sample_range_uniform(-PI, PI);
        let height = self.sample_range_uniform(-1.0, 1.0);

        let r = (1.0 - height * height as f32).sqrt();

        let sample = Vec3 {
            x: r * azimuth.cos(),
            y: height,
            // z: -r * azimuth.sin(),
            z: 0.0,
        };

        sample.as_normal()
    }
}

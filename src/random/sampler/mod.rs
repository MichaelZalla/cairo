use std::f32::consts::PI;

use rand::rngs::ThreadRng;

use rand_distr::{Distribution, Normal, NormalError, Uniform};

use distribution::GeneratedDistribution;

use crate::{matrix::Mat4, vec::vec3::Vec3};

pub mod distribution;

pub trait RangeSampler {
    fn sample_range_uniform(&mut self, min: f32, max: f32) -> f32;
    fn sample_range_normal(&mut self, mean: f32, std_dev: f32) -> f32;
}

pub trait DirectionSampler {
    fn sample_direction_uniform(&mut self) -> Vec3;
}

pub trait VectorDisplaceSampler {
    fn sample_displacement_uniform(&mut self, v: &Vec3, max_deflection_angle_radians: f32) -> Vec3;
    fn sample_displacement_normal(
        &mut self,
        v: &Vec3,
        max_deflection_angle_radians: f32,
    ) -> Result<Vec3, NormalError>;
}

#[derive(Default, Debug, Clone)]
pub struct RandomSampler<const N: usize> {
    rng: ThreadRng,
    uniform_seed: GeneratedDistribution<N>,
    normal_seed: GeneratedDistribution<N>,
    is_seeded: bool,
}

impl<const N: usize> RandomSampler<N> {
    pub fn seed(&mut self) -> Result<(), NormalError> {
        let uniform_sampler = Uniform::new_inclusive(0.0, 1.0).unwrap();

        let normal_sampler = Normal::new(0.0, 1.0)?;

        for i in 0..N {
            self.uniform_seed.values[i] = uniform_sampler.sample(&mut self.rng);
            self.normal_seed.values[i] = normal_sampler.sample(&mut self.rng);
        }

        self.is_seeded = true;

        Ok(())
    }

    // Returns a uniformly distributed random scalar in the range [min...max].
    fn _sample_range_uniform(&mut self, min: f32, max: f32) -> f32 {
        let sampler = Uniform::new_inclusive(min, max).unwrap();

        sampler.sample(&mut self.rng)
    }

    /// Returns a normally distributed random scalar with a given mean and
    /// standard deviation.
    fn _sample_range_normal(&mut self, mean: f32, std_dev: f32) -> Result<f32, NormalError> {
        match Normal::new(mean, std_dev) {
            Ok(distribution) => Ok(distribution.sample(&mut self.rng)),
            Err(err) => Err(err),
        }
    }

    fn sample_displacement(&mut self, v: &Vec3, max_deflection_angle_radians: f32, f: f32) -> Vec3 {
        // See: https://math.stackexchange.com/a/4343075

        let (normal, tangent, bitangent) = v.basis();

        let basis = Mat4::tbn(tangent, bitangent, normal);

        // Rotation currently only happens in the positive direction.

        let phi = f.sqrt() * max_deflection_angle_radians;

        let theta = self.sample_range_uniform(-PI, PI);

        let v_hat_prime = Vec3 {
            x: theta.cos() * theta.sin(),
            y: theta.sin() * phi.sin(),
            z: phi.cos(),
        };

        (v_hat_prime * basis) * v.mag()
    }
}

impl<const N: usize> RangeSampler for RandomSampler<N> {
    // Returns a uniformly distributed random scalar in the range [min...max].
    fn sample_range_uniform(&mut self, min: f32, max: f32) -> f32 {
        let sample = self.uniform_seed.sample();

        (max - min) * sample + min
    }

    /// Returns a normally distributed random scalar with a given mean and
    /// standard deviation.
    fn sample_range_normal(&mut self, mean: f32, std_dev: f32) -> f32 {
        let sample = self.normal_seed.sample();

        std_dev * sample + mean
    }
}

impl<const N: usize> DirectionSampler for RandomSampler<N> {
    // Returns a uniformly random normal on the unit circle.
    fn sample_direction_uniform(&mut self) -> Vec3 {
        let azimuth = self.sample_range_uniform(-PI, PI);
        let height = self.sample_range_uniform(-1.0, 1.0);

        let r = (1.0 - height * height).sqrt();

        let sample = Vec3 {
            x: r * azimuth.cos(),
            y: height,
            z: -r * azimuth.sin(),
        };

        sample.as_normal()
    }
}

impl<const N: usize> VectorDisplaceSampler for RandomSampler<N> {
    fn sample_displacement_uniform(&mut self, v: &Vec3, max_deflection_angle_radians: f32) -> Vec3 {
        let f = self.sample_range_uniform(0.0, 1.0);

        self.sample_displacement(v, max_deflection_angle_radians, f)
    }

    fn sample_displacement_normal(
        &mut self,
        v: &Vec3,
        max_deflection_angle_radians: f32,
    ) -> Result<Vec3, NormalError> {
        let std_dev = max_deflection_angle_radians / 3.0;

        let f = self.sample_range_normal(0.0, std_dev);

        Ok(self.sample_displacement(v, max_deflection_angle_radians, f.abs()))
    }
}

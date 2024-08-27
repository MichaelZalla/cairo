use std::f32::consts::PI;

use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};

use cairo::{
    matrix::Mat4,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};
use rand_distr::{Normal, NormalError};

#[derive(Default, Debug, Clone)]
pub struct RandomSampler {
    rng: ThreadRng,
}

pub trait RangeSampler {
    fn sample_range_uniform(&mut self, min: f32, max: f32) -> f32;
    fn sample_range_normal(&mut self, mean: f32, std_dev: f32) -> Result<f32, NormalError>;
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

impl RandomSampler {
    fn sample_displacement(&mut self, v: &Vec3, max_deflection_angle_radians: f32, f: f32) -> Vec3 {
        // See: https://math.stackexchange.com/a/4343075

        let normal = v.as_normal();
        let tangent = vec3::UP.cross(normal).as_normal();
        let bitangent = normal.cross(tangent);

        // @NOTE: Using {normal, bitangent, tangent} order such that `normal`
        // becomes the X-axis in our new frame of reference; for 3D, we will
        // want `normal` to serve as the Z-axis instead.
        let basis = Mat4::tbn(normal, bitangent, tangent);

        let phi = f.sqrt() * max_deflection_angle_radians;

        // @NOTE: Skipping rotation sample for now, as we don't need it for 2D.
        // let theta = self.sample_range_uniform(-PI, PI);

        let right_rotated = Vec4::new(vec3::RIGHT, 1.0) * Mat4::rotation_z(phi);

        ((right_rotated * basis) * v.mag()).to_vec3()
    }
}

impl RangeSampler for RandomSampler {
    // Returns a uniformly distributed random scalar in the range [min...max].
    fn sample_range_uniform(&mut self, min: f32, max: f32) -> f32 {
        let sampler = Uniform::new_inclusive(min, max);

        sampler.sample(&mut self.rng)
    }

    /// Returns a normally distributed random scalar with a given mean and
    /// standard deviation.
    fn sample_range_normal(&mut self, mean: f32, std_dev: f32) -> Result<f32, NormalError> {
        match Normal::new(mean, std_dev) {
            Ok(distribution) => Ok(distribution.sample(&mut self.rng)),
            Err(err) => Err(err),
        }
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

impl VectorDisplaceSampler for RandomSampler {
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

        match self.sample_range_normal(0.0, std_dev) {
            Ok(f) => Ok(self.sample_displacement(v, max_deflection_angle_radians, f.abs())),
            Err(err) => Err(err),
        }
    }
}

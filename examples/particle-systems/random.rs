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

pub trait VectorDisplaceSampler {
    fn sample_displacement_uniform(&mut self, v: &Vec3, max_deflection_angle_radians: f32) -> Vec3;
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

impl VectorDisplaceSampler for RandomSampler {
    fn sample_displacement_uniform(&mut self, v: &Vec3, max_deflection_angle_radians: f32) -> Vec3 {
        // See: https://math.stackexchange.com/a/4343075

        let normal = v.as_normal();
        let tangent = vec3::UP.cross(normal).as_normal();
        let bitangent = normal.cross(tangent);

        // @NOTE: Using {normal, bitangent, tangent} order such that `normal`
        // becomes the X-axis in our new frame of reference; for 3D, we will
        // want `normal` to serve as the Z-axis instead.
        let basis = Mat4::tbn(normal, bitangent, tangent);

        // let f = self.sample_range_uniform(0.0, 1.0);
        let f =
            self.sample_range_uniform(-max_deflection_angle_radians, max_deflection_angle_radians);

        // let phi = f.sqrt() * max_deflection_angle_radians;
        let phi = f;

        // @NOTE: Skipping rotation sample for now, as we don't need it for 2D.
        // let theta = self.sample_range_uniform(-PI, PI);

        let right_rotated = Vec4::new(vec3::RIGHT, 1.0) * Mat4::rotation_z(phi);

        ((right_rotated * basis) * v.mag()).to_vec3()
    }
}

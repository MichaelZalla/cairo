use rand_distr::{Distribution, Uniform};

use crate::{animation::lerp, vec::vec3::Vec3};

pub(in crate::software_renderer) const KERNEL_SIZE: usize = 64;

pub(in crate::software_renderer) fn make_hemisphere_kernel() -> [Vec3; KERNEL_SIZE] {
    let mut rng = rand::thread_rng();

    let uniform = Uniform::<f32>::new(0.0, 1.0);

    let mut hemisphere_kernel: [Vec3; KERNEL_SIZE] = [Default::default(); KERNEL_SIZE];

    for i in 0..KERNEL_SIZE {
        let mut scale = i as f32 / KERNEL_SIZE as f32;

        scale = lerp(0.1, 1.0, scale * scale);

        let half_box_sample = Vec3 {
            x: uniform.sample(&mut rng) * 2.0 - 1.0,
            y: uniform.sample(&mut rng) * 2.0 - 1.0,
            z: uniform.sample(&mut rng), // Forward, in tangent space.
        };

        let hemisphere_sample = half_box_sample.as_normal() * scale;

        hemisphere_kernel[0] = hemisphere_sample;
    }

    hemisphere_kernel
}

pub(in crate::software_renderer) fn make_4x4_tangent_space_rotations() -> [Vec3; 16] {
    let mut rng = rand::thread_rng();

    let uniform = Uniform::<f32>::new(0.0, 1.0);

    let mut noise_samples = [Default::default(); 16];

    for sample in noise_samples.iter_mut() {
        *sample = Vec3 {
            x: uniform.sample(&mut rng) * 2.0 - 1.0,
            y: uniform.sample(&mut rng) * 2.0 - 1.0,
            z: 0.0,
        };
    }

    noise_samples
}

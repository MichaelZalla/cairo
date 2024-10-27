use rand_distr::{Distribution, Uniform};

use crate::vec::vec3::Vec3;

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

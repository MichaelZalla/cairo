#![allow(non_upper_case_globals)]

use std::f32::consts::{PI, TAU};

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub static HdrDiffuseIrradianceFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        let handle = shader_context.ambient_radiance_map.unwrap();

        match resources.cubemap_vec3.borrow().get(&handle) {
            Ok(entry) => {
                let map = &entry.item;

                let normal = sample.position_world_space.as_normal();

                let mut irradiance = Default::default();

                static SAMPLE_DELTA: f32 = 0.05;

                let mut sample_count: usize = 0;

                // Convolution.

                let mut up = Vec3 {
                    y: 1.0,
                    ..Default::default()
                };

                let right = up.cross(normal).as_normal();

                up = normal.cross(right).as_normal();

                let mut phi = 0.0;

                while phi < TAU {
                    let mut theta = 0.0;

                    while theta < 0.5 * PI {
                        // Spherical to cartesian (in tangent space).

                        let cartesian = Vec3 {
                            x: theta.sin() * phi.cos(),
                            y: theta.sin() * phi.sin(),
                            z: theta.cos(),
                        };

                        // Tangent space to world position.

                        let world_pos =
                            right * cartesian.x + up * cartesian.y + normal * cartesian.z;

                        // Sampled radiance, scaled by the incoming light angle (theta).

                        // "Note that we scale the sampled color value by
                        // cos(theta) due to the light being weaker at larger
                        // angles and by sin(theta) to account for the smaller
                        // sample areas in the higher hemisphere areas."

                        let radiance =
                            map.sample_nearest(&Vec4::new(world_pos, 1.0), None)/* * theta.cos() * theta.sin()*/;

                        irradiance += radiance;

                        sample_count += 1;

                        theta += SAMPLE_DELTA;
                    }

                    phi += SAMPLE_DELTA;
                }

                irradiance = irradiance * PI * (1.0 / sample_count as f32);

                irradiance
            }
            Err(_) => panic!(),
        }
    };

pub static HdrDiffuseRadianceCubemapFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        let handle = shader_context.ambient_diffuse_irradiance_map.unwrap();

        match resources.cubemap_vec3.borrow().get(&handle) {
            Ok(entry) => {
                let map = &entry.item;

                let normal = sample.position_world_space.as_normal();

                let irradiance = map.sample_nearest(&Vec4::new(normal, 0.0), None);

                #[allow(clippy::let_and_return)]
                irradiance
            }
            Err(_) => panic!(),
        }
    };

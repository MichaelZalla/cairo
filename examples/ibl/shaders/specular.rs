#![allow(non_upper_case_globals)]

use cairo::{
    color::{self, Color},
    physics::pbr::importance_sample_ggx,
    random::hammersley_2d_sequence,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub const HdrSpecularPrefilteredEnvironmentFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext,
     resources: &SceneResources,
     sample: &GeometrySample|
     -> Color {
        if let Some(handle) = shader_context.active_ambient_radiance_map {
            if let Ok(entry) = resources.cubemap_vec3.borrow().get(&handle) {
                let map = &entry.item;

                // Assumes the fragment-to-view direction (and thus the direction of the
                // specular reflection) to be identical to surface normal direction.

                let normal = sample.world_pos.as_normal();

                let direction_to_view_position = normal;

                static SAMPLE_COUNT: usize = 1024;

                let mut prefiltered_irradiance: Vec3 = Default::default();
                let mut total_weight = 0.0;

                let one_over_n = 1.0 / SAMPLE_COUNT as f32;

                for i in 0..SAMPLE_COUNT {
                    let random_direction_hammersley = hammersley_2d_sequence(i as u32, one_over_n);

                    let biased_sample_direction = importance_sample_ggx(
                        random_direction_hammersley,
                        &normal,
                        sample.roughness,
                    );

                    let direction_to_environment_light = (biased_sample_direction
                        * (2.0 * direction_to_view_position.dot(biased_sample_direction))
                        - direction_to_view_position)
                        .as_normal();

                    let likeness_to_environment_light =
                        normal.dot(direction_to_environment_light).max(0.0);

                    if likeness_to_environment_light > 0.0 {
                        prefiltered_irradiance += map
                            .sample_nearest(&Vec4::new(direction_to_environment_light, 1.0), None)
                            * likeness_to_environment_light;

                        total_weight += likeness_to_environment_light;
                    }
                }

                prefiltered_irradiance /= total_weight;

                return Color::from_vec3(prefiltered_irradiance);
            }
        }

        color::GREEN
    };

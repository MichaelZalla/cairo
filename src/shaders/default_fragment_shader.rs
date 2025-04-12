use crate::{
    animation::lerp,
    physics::pbr::brdf::fresnel_schlick_indirect,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    texture::{cubemap::CubeMap, map::TextureMap, sample::sample_nearest_vec2},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

pub static DEFAULT_FRAGMENT_SHADER: FragmentShaderFn =
    |context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Surface reflection at zero incidence.
        #[allow(non_upper_case_globals)]
        static f0_dielectic: Vec3 = Vec3 {
            x: 0.04,
            y: 0.04,
            z: 0.04,
        };

        let f0_metal = sample.albedo;

        let f0 = lerp(f0_dielectic, f0_metal, sample.metallic);

        // Calculate ambient light contribution.

        let ambient_light_contribution = match (
            &context.ambient_diffuse_irradiance_map,
            &context.ambient_specular_prefiltered_environment_map,
            &context.ambient_specular_brdf_integration_map,
        ) {
            (
                Some(diffuse_irradiance_map_handle),
                Some(specular_prefiltered_environment_map_handle),
                Some(specular_brdf_integration_map_handle),
            ) => {
                match (
                    resources
                        .cubemap_vec3
                        .borrow()
                        .get(diffuse_irradiance_map_handle),
                    resources
                        .cubemap_vec3
                        .borrow()
                        .get(specular_prefiltered_environment_map_handle),
                    resources
                        .texture_vec2
                        .borrow()
                        .get(specular_brdf_integration_map_handle),
                ) {
                    (
                        Ok(diffuse_irradiance_map_entry),
                        Ok(specular_prefiltered_environment_map_entry),
                        Ok(specular_brdf_integration_map_entry),
                    ) => {
                        let diffuse_irradiance_map = &diffuse_irradiance_map_entry.item;

                        let specular_prefiltered_environment_map =
                            &specular_prefiltered_environment_map_entry.item;

                        let specular_brdf_integration_map =
                            &specular_brdf_integration_map_entry.item;

                        contribute_ambient_ibl(
                            context,
                            diffuse_irradiance_map,
                            specular_prefiltered_environment_map,
                            specular_brdf_integration_map,
                            sample,
                            &f0,
                        )
                    }
                    _ => panic!("Failed to get CubeMap from Arena."),
                }
            }
            _ => match &context.ambient_light {
                Some(handle) => match resources.ambient_light.borrow().get(handle) {
                    Ok(entry) => {
                        let light = &entry.item;

                        light.contribute_pbr(sample)
                    }
                    Err(err) => panic!(
                        "Failed to get AmbientLight from Arena: {:?}: {}",
                        handle, err
                    ),
                },
                None => Default::default(),
            },
        };

        // Calculate directional light contribution.

        let directional_light_contribution = match &context.directional_light {
            Some(handle) => {
                let texture_f32_arena = resources.texture_f32.borrow();
                let directional_light_arena = resources.directional_light.borrow();

                match directional_light_arena.get(handle) {
                    Ok(entry) => {
                        let light = &entry.item;

                        light.contribute_pbr(
                            sample,
                            &f0,
                            &texture_f32_arena,
                            context,
                            light.shadow_maps.as_ref(),
                        )
                    }
                    Err(err) => panic!(
                        "Failed to get DirectionalLight from Arena: {:?}: {}",
                        handle, err
                    ),
                }
            }
            None => Default::default(),
        };

        // Calculate point light contributions.

        let mut point_light_contribution: Vec3 = Default::default();

        for handle in &context.point_lights {
            match resources.point_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    if let Some(handle) = light.shadow_map {
                        if let Ok(entry) = resources.cubemap_f32.borrow().get(&handle) {
                            let shadow_map = &entry.item;

                            point_light_contribution +=
                                light.contribute_pbr(sample, &f0, Some(shadow_map));
                        } else {
                            point_light_contribution += light.contribute_pbr(sample, &f0, None);
                        }
                    } else {
                        point_light_contribution += light.contribute_pbr(sample, &f0, None);
                    }
                }
                Err(err) => panic!("Failed to get PointLight from Arena: {:?}: {}", handle, err),
            }
        }

        // Calculate spot light contributions.

        let mut spot_light_contribution: Vec3 = Default::default();

        for handle in &context.spot_lights {
            match resources.spot_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    if let Some(handle) = light.shadow_map {
                        if let Ok(entry) = resources.texture_f32.borrow().get(&handle) {
                            let shadow_map = &entry.item;

                            spot_light_contribution += light.contribute_pbr(
                                sample,
                                &f0,
                                &context.view_position,
                                Some(shadow_map),
                            );
                        } else {
                            spot_light_contribution +=
                                light.contribute_pbr(sample, &f0, &context.view_position, None);
                        }
                    } else {
                        spot_light_contribution +=
                            light.contribute_pbr(sample, &f0, &context.view_position, None);
                    }
                }
                Err(err) => panic!("Failed to get SpotLight from Arena: {:?}: {}", handle, err),
            }
        }

        // Calculate emissive light contribution.

        let emissive_light_contribution: Vec3 = sample.emissive_color;

        // Combine light intensities.

        ambient_light_contribution
            + directional_light_contribution
            + point_light_contribution
            + spot_light_contribution
            + emissive_light_contribution
    };

fn contribute_ambient_ibl(
    context: &ShaderContext,
    diffuse_irradiance_map: &CubeMap<Vec3>,
    specular_prefiltered_environment_map: &CubeMap<Vec3>,
    specular_brdf_integration_map: &TextureMap<Vec2>,
    sample: &GeometrySample,
    f0: &Vec3,
) -> Vec3 {
    // Total incoming ambient light from environment.

    let cubemap_rotation_transform = context.skybox_transform.unwrap_or_default();

    let irradiance = diffuse_irradiance_map.sample_nearest(
        &(Vec4::new(sample.normal_world_space, 1.0) * cubemap_rotation_transform),
        None,
    );

    let normal = sample.tangent_space_info.normal;

    let fragment_to_view_tangent_space =
        sample.tangent_space_info.view_position - sample.tangent_space_info.fragment_position;

    let direction_to_view_position = fragment_to_view_tangent_space.as_normal();

    let normal_likeness_to_view_direction = normal.dot(direction_to_view_position).max(0.0);

    // Ratio of reflected light energy.

    let fresnel = fresnel_schlick_indirect(normal_likeness_to_view_direction, f0, sample.roughness);

    let specular_prefiltered_environment_irradiance = {
        static MAX_LOD_FOR_PREFILTERED_ENVIRONMENT_MAP: f32 = 4.0;

        let specular_prefiltered_environment_lod: f32 =
            sample.roughness * MAX_LOD_FOR_PREFILTERED_ENVIRONMENT_MAP;

        let normal_world_space = sample.normal_world_space;

        let view_position_world_space = context.view_position;

        let fragment_to_view = view_position_world_space.to_vec3() - sample.position_world_space;

        let reflected_ray_direction = (fragment_to_view.as_normal()).reflect(normal_world_space);

        let near_level_index = specular_prefiltered_environment_lod.floor() as usize;

        let far_level_index = near_level_index + 1;

        let alpha =
            specular_prefiltered_environment_lod - (specular_prefiltered_environment_lod.floor());

        specular_prefiltered_environment_map.sample_trilinear(
            &(Vec4::new(reflected_ray_direction, 1.0) * cubemap_rotation_transform),
            near_level_index,
            far_level_index,
            alpha,
        )
    };

    let specular_brdf_response = {
        let uv = Vec2 {
            x: normal_likeness_to_view_direction,
            y: 1.0 - sample.roughness,
            z: 0.0,
        };

        sample_nearest_vec2(uv, specular_brdf_integration_map, None)
    };

    let indirect_specular_irradiance = specular_prefiltered_environment_irradiance
        * (fresnel * specular_brdf_response.x + specular_brdf_response.y);

    let specular = indirect_specular_irradiance;

    let k_s = fresnel;

    // Ratio of refracted light energy (scaled by metallic).

    let k_d = (vec3::ONES - k_s) * (1.0 - sample.metallic);

    let indirect_diffuse_irradiance = irradiance * sample.albedo;

    (k_d * indirect_diffuse_irradiance + specular) * sample.ambient_factor
}

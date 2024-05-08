use crate::{
    animation::lerp,
    color::Color,
    physics::pbr::fresnel_schlick_roughness,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

pub static DEFAULT_FRAGMENT_SHADER: FragmentShaderFn =
    |context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Surface reflection at zero incidence.
        #[allow(non_upper_case_globals)]
        static f0_dielectic: Vec3 = Vec3 {
            x: 0.04,
            y: 0.04,
            z: 0.04,
        };

        let f0_metal = sample.albedo;

        let f0 = lerp(f0_dielectic, f0_metal, sample.metallic);

        // Calculate ambient light contribution

        let ambient_light_contribution = match &context.active_ambient_diffuse_map {
            Some(handle) => match resources.skybox_hdr.borrow().get(handle) {
                Ok(entry) => {
                    let map = &entry.item;

                    let fragment_to_view_tangent_space = sample.tangent_space_info.view_position
                        - sample.tangent_space_info.fragment_position;

                    let view_direction_normal = fragment_to_view_tangent_space.as_normal();

                    let likeness = sample
                        .tangent_space_info
                        .normal
                        .dot(view_direction_normal)
                        .max(0.0);

                    // Rendering equation

                    let fresnel = fresnel_schlick_roughness(likeness, &f0, sample.roughness);

                    // Ratio of reflected light energy.
                    let k_s = fresnel;

                    // Ratio of refracted light energy.
                    let k_d = (vec3::ONES - k_s) * (1.0 - sample.metallic);

                    let irradiance = map.sample(&Vec4::new(sample.world_normal, 1.0));

                    let diffuse = irradiance * sample.albedo;

                    (k_d * diffuse) * sample.ambient_factor
                }
                Err(err) => panic!("Failed to get CubeMap from Arena: {:?}: {}", handle, err),
            },
            None => match &context.ambient_light {
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

        // Calculate directional light contribution

        let directional_light_contribution = match &context.directional_light {
            Some(handle) => match resources.directional_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    light.contribute_pbr(sample, &f0)
                }
                Err(err) => panic!(
                    "Failed to get DirectionalLight from Arena: {:?}: {}",
                    handle, err
                ),
            },
            None => Default::default(),
        };

        // Calculate point light contributions (including specular)

        let mut point_light_contribution: Vec3 = Default::default();

        for handle in &context.point_lights {
            match resources.point_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    point_light_contribution += light.contribute_pbr(sample, &f0);
                }
                Err(err) => panic!("Failed to get PointLight from Arena: {:?}: {}", handle, err),
            }
        }

        // Calculate spot light contributions (including specular).

        let mut spot_light_contribution: Vec3 = Default::default();

        for handle in &context.spot_lights {
            match resources.spot_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    spot_light_contribution += light.contribute_pbr(sample, &f0);
                }
                Err(err) => panic!("Failed to get SpotLight from Arena: {:?}: {}", handle, err),
            }
        }

        // Calculate emissive light contribution

        let emissive_light_contribution: Vec3 = sample.emissive_color;

        // Combine light intensities

        let total_contribution = ambient_light_contribution
            + directional_light_contribution
            + point_light_contribution
            + spot_light_contribution
            + emissive_light_contribution;

        Color::from_vec3(total_contribution)
    };

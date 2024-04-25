use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::Vec3,
};

pub static DEFAULT_FRAGMENT_SHADER: FragmentShaderFn =
    |context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Calculate ambient light contribution

        let ambient_light_contribution = match &context.ambient_light {
            Some(handle) => match resources.ambient_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    light.contribute(sample)
                }
                Err(err) => panic!(
                    "Failed to get AmbientLight from Arena: {:?}: {}",
                    handle, err
                ),
            },
            None => Default::default(),
        };

        // Calculate directional light contribution

        let directional_light_contribution = match &context.directional_light {
            Some(handle) => match resources.directional_light.borrow().get(handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    light.contribute(sample)
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

                    point_light_contribution += light.contribute(sample);
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

                    spot_light_contribution += light.contribute(sample.world_pos);
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

        // @TODO Honor each material's ambient, diffuse, and specular colors.

        let mut color: Vec3 = sample.diffuse_color;

        // Transform sRGB space to linear space.

        color.srgb_to_linear();

        // Multiply by total lighting contribution and saturate.

        color *= total_contribution;

        Color::from_vec3(color)
    };

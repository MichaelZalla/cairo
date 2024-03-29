use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::Vec3,
};

pub static DEFAULT_FRAGMENT_SHADER: FragmentShaderFn =
    |context: &ShaderContext, sample: &GeometrySample| -> Color {
        // Calculate ambient light contribution

        let ambient_light_contribution = match context.ambient_light {
            Some(light) => light.contribute(sample.ambient_factor),
            None => Default::default(),
        };

        // Calculate directional light contribution

        let directional_light_contribution = match context.directional_light {
            Some(light) => light.contribute(sample.normal),
            None => Default::default(),
        };

        // Calculate point light contributions (including specular)

        let mut point_light_contribution: Vec3 = Default::default();

        for point_light in &context.point_lights {
            point_light_contribution += point_light.contribute(sample);
        }

        // Calculate spot light contributions (including specular).

        let mut spot_light_contribution: Vec3 = Default::default();

        for spot_light in &context.spot_lights {
            spot_light_contribution += spot_light.contribute(sample.world_pos);
        }

        // Calculate emissive light contribution

        let emissive_light_contribution: Vec3 = sample.emissive;

        // Combine light intensities

        let total_contribution = ambient_light_contribution
            + directional_light_contribution
            + point_light_contribution
            + spot_light_contribution
            + emissive_light_contribution;

        // @TODO Honor each material's ambient, diffuse, and specular colors.

        let mut color: Vec3 = sample.diffuse;

        // Transform sRGB space to linear space.

        color.srgb_to_linear();

        // Multiply by total lighting contribution and saturate.

        color *= total_contribution;

        Color::from_vec3(color)
    };

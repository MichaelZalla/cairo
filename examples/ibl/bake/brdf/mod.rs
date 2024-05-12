use cairo::{
    buffer::Buffer2D,
    physics::pbr::{brdf::geometry_smith_indirect, sampling::importance_sample_ggx},
    random::hammersley_2d_sequence,
    texture::map::TextureMap,
    vec::{vec2::Vec2, vec3::Vec3},
};

pub fn generate_specular_brdf_integration_map(size: u32) -> TextureMap<Vec2> {
    let mut map = TextureMap::from_buffer(size, size, Buffer2D::<Vec2>::new(512, 512, None));

    // Integrate specular BRDF over angle and roughness (axes).

    let one_over_size_doubled = 1.0 / (size as f32) / 2.0;

    for y in 0..size {
        let y_alpha = one_over_size_doubled / 2.0 + y as f32 / (size + 1) as f32;

        for x in 0..size {
            let x_alpha = one_over_size_doubled / 2.0 + x as f32 / (size + 1) as f32;

            let likeness_to_view_direction = x_alpha;
            let roughness = y_alpha;

            map.levels[0].0.set(
                x,
                size - 1 - y,
                integrate_brdf(likeness_to_view_direction, roughness),
            );
        }
    }

    map
}

fn integrate_brdf(normal_likeness_to_view_direction: f32, roughness: f32) -> Vec2 {
    let direction_to_view_position = Vec3 {
        x: (1.0 - normal_likeness_to_view_direction * normal_likeness_to_view_direction).sqrt(),
        y: 0.0,
        z: normal_likeness_to_view_direction,
    };

    let mut accumulated_scale: f32 = 0.0;
    let mut accumulated_bias: f32 = 0.0;

    let normal = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.999,
    };

    static SAMPLE_COUNT: usize = 1024;

    let one_over_n = 1.0 / SAMPLE_COUNT as f32;

    for i in 0..SAMPLE_COUNT {
        let random_direction_hammersley = hammersley_2d_sequence(i as u32, one_over_n);

        let biased_sample_direction =
            importance_sample_ggx(random_direction_hammersley, &normal, roughness);

        let direction_to_environment_light = (biased_sample_direction
            * (2.0 * direction_to_view_position.dot(biased_sample_direction))
            - direction_to_view_position)
            .as_normal();

        let normal_likeness_to_environment_light = (direction_to_environment_light.z).max(0.0);

        let normal_likeness_to_biased_sample_direction = (biased_sample_direction.z).max(0.0);

        let view_likeness_to_biased_sample_direction = direction_to_view_position
            .dot(biased_sample_direction)
            .max(0.0);

        if normal_likeness_to_environment_light > 0.0
            && normal_likeness_to_biased_sample_direction > 0.0
        {
            let g = geometry_smith_indirect(
                &normal,
                &direction_to_view_position,
                &direction_to_environment_light,
                roughness,
            );

            let g_vis = (g * view_likeness_to_biased_sample_direction)
                / (normal_likeness_to_biased_sample_direction * normal_likeness_to_view_direction);

            let fc = (1.0 - view_likeness_to_biased_sample_direction).powi(5);

            accumulated_scale += (1.0 - fc) * g_vis;
            accumulated_bias += fc * g_vis;
        }
    }

    let scale = accumulated_scale / SAMPLE_COUNT as f32;
    let bias = accumulated_bias / SAMPLE_COUNT as f32;

    Vec2 {
        x: scale,
        y: bias,
        z: 0.0,
    }
}

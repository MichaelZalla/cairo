#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub static DirectionalShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Color {
        let p = Vec4::new(sample.world_pos, 1.0)
            * context.view_inverse_transform
            * context.projection_transform;

        let directional_light_arena = resources.directional_light.borrow();

        let (depth, _max_depth) =
            match directional_light_arena.get(&context.directional_light.unwrap()) {
                Ok(entry) => {
                    let directional_light = &entry.item;

                    let cameras = directional_light.shadow_map_cameras.as_ref().unwrap();
                    let camera_index = context.directional_light_view_projection_index.unwrap();

                    let (_far_z, camera) = cameras[camera_index];

                    let (near, far) = (
                        camera.get_projection_z_near(),
                        camera.get_projection_z_far(),
                    );

                    let max_depth = far - near;

                    let depth = p.z.max(near).min(far);

                    (depth, max_depth)
                }
                Err(_) => panic!(),
            };

        if depth > 1.0 {
            Default::default()
        } else {
            let distance_alpha = depth / 100.0;

            Color::from_vec3(Vec3::ones() * distance_alpha)
        }
    };

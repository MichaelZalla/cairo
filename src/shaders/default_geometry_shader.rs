use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext,
        geometry::{options::GeometryShaderOptions, sample::GeometrySample, GeometryShaderFn},
    },
    texture::sample::{sample_bilinear, sample_nearest},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

pub static DEFAULT_GEOMETRY_SHADER: GeometryShaderFn = |context: &ShaderContext,
                                                        resources: &SceneResources,
                                                        options: &GeometryShaderOptions,
                                                        interpolant: &DefaultVertexOut|
 -> Option<GeometrySample> {
    let mut out = GeometrySample {
        stencil: true,
        uv: interpolant.uv,
        world_normal: interpolant.normal.to_vec3(),
        tangent_space_info: interpolant.tangent_space_info,
        world_pos: interpolant.world_pos,
        depth: interpolant.depth,
        ambient_factor: 1.0,
        diffuse_color: Vec3::ones(),
        specular_color: Default::default(),
        specular_exponent: 8,
        emissive_color: Default::default(),
        alpha: 1.0,
    };

    if let Some(name) = &context.active_material {
        match resources.material.borrow().get(name) {
            Some(material) => {
                // Surface normal mapping.
                if let (Some(handle), true) = (&material.normal_map, options.normal_mapping_active)
                {
                    match resources.texture.borrow().get(handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, g, b) = sample_nearest(out.uv, map, None);

                            // Map the normal's components into the range [-1, 1].

                            // @TODO Flip y axis for standard normal maps.
                            let tangent_space_normal = Vec4 {
                                x: (r as f32 / 255.0) * 2.0 - 1.0,
                                y: (g as f32 / 255.0) * 2.0 - 1.0,
                                z: (b as f32 / 255.0) * 2.0 - 1.0,
                                w: 1.0,
                            };

                            // Perturb the surface normal using the local
                            // tangent-space information read from `map`.

                            out.world_normal = (tangent_space_normal
                                * interpolant.tangent_space_info.tbn)
                                .to_vec3()
                                .as_normal();

                            out.tangent_space_info.normal =
                                tangent_space_normal.to_vec3().as_normal();
                        }
                        Err(err) => {
                            panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                        }
                    }
                }

                // Ambient occlusion mapping.
                if let (Some(handle), true) = (
                    &material.ambient_occlusion_map,
                    options.ambient_occlusion_mapping_active,
                ) {
                    match resources.texture.borrow().get(handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = sample_nearest(out.uv, map, None);

                            out.ambient_factor = r as f32 / 255.0;
                        }
                        Err(err) => {
                            panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                        }
                    }
                }

                // Diffuse color
                match (
                    &material.diffuse_color_map,
                    options.diffuse_color_mapping_active,
                ) {
                    (Some(handle), true) => match resources.texture.borrow().get(handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, g, b) = if options.bilinear_active {
                                sample_bilinear(out.uv, map, None)
                            } else {
                                sample_nearest(out.uv, map, None)
                            };

                            let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                            color.srgb_to_linear();

                            out.diffuse_color = color;
                        }
                        Err(err) => {
                            panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                        }
                    },
                    _ => {
                        // No diffuse map defined for this material, or
                        // diffuse mapping is disabled.

                        out.diffuse_color = material.diffuse_color;
                    }
                }

                // Specular color
                match (
                    &material.specular_color_map,
                    options.specular_exponent_mapping_active,
                ) {
                    (Some(handle), true) => match &resources.texture.borrow().get(handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, g, b) = sample_nearest(out.uv, map, None);

                            let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                            color.srgb_to_linear();

                            out.specular_color = color;
                        }
                        Err(_) => panic!("Invalid TextureMap handle!"),
                    },
                    _ => {
                        out.specular_color = material.specular_color;
                    }
                }

                // Specular exponent
                match (
                    &material.specular_exponent_map,
                    options.specular_exponent_mapping_active,
                ) {
                    (Some(handle), true) => match &resources.texture.borrow().get(handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = sample_nearest(out.uv, map, None);

                            out.specular_exponent = r as i32;
                        }
                        Err(_) => panic!("Invalid TextureMap handle!"),
                    },
                    _ => {
                        out.specular_exponent = material.specular_exponent;
                    }
                }

                // Emissive color
                match material.emissive_color_map {
                    Some(handle) => match resources.texture.borrow().get(&handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, g, b) = sample_nearest(out.uv, map, None);

                            let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                            color.srgb_to_linear();

                            out.emissive_color = color;
                        }
                        Err(err) => {
                            panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                        }
                    },
                    None => {
                        out.emissive_color = material.emissive_color;
                    }
                }

                // Alpha transparency
                match material.alpha_map {
                    Some(handle) => match resources.texture.borrow().get(&handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = sample_nearest(out.uv, map, None);

                            out.alpha = r as f32 / 255.0;
                        }
                        Err(err) => {
                            panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                        }
                    },
                    None => {
                        out.alpha = 1.0 - material.transparency;
                    }
                }

                // Displacement (height)
                if let (Some(handle), true) = (
                    &material.displacement_map,
                    options.displacement_mapping_active,
                ) {
                    match resources.texture.borrow().get(handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = sample_nearest(interpolant.uv, map, None);

                            let displacement = r as f32 / 255.0;

                            // Modify sample UV based on height map, if
                            // necessary, before proceeding.

                            static LAYER_COUNT_MIN: f32 = 8.0;
                            static LAYER_COUNT_MAX: f32 = 32.0;

                            static Z_FORWARD_TANGENT_SPACE: Vec3 = vec3::FORWARD;

                            let get_parallax_mapped_uv =
                                |uv: Vec2,
                                 fragment_to_view_direction_tangent_space: Vec3,
                                 displacement: f32|
                                 -> Vec2 {
                                    // Scale the view-direction vector (in tangent
                                    // space) by the sampled displacement, modulated
                                    // by a scaling factor.

                                    let alpha = Z_FORWARD_TANGENT_SPACE
                                        .dot(fragment_to_view_direction_tangent_space)
                                        .max(0.0);

                                    let layer_count = (LAYER_COUNT_MAX
                                        - (LAYER_COUNT_MAX - LAYER_COUNT_MIN) * alpha)
                                        .floor();

                                    let layer_depth: f32 = 1.0 / layer_count;

                                    let p = Vec2 {
                                        x: fragment_to_view_direction_tangent_space.x
                                            / fragment_to_view_direction_tangent_space.z,
                                        y: fragment_to_view_direction_tangent_space.y
                                            / fragment_to_view_direction_tangent_space.z,
                                        z: 1.0,
                                    } * displacement
                                        * material.displacement_scale;

                                    let uv_step = p / layer_count;

                                    let mut current_layer_depth = 0.0;
                                    let mut current_uv = uv;
                                    let mut current_sampled_displacement = displacement;

                                    while current_layer_depth < current_sampled_displacement {
                                        // Take a step along P.
                                        current_uv -= uv_step;

                                        // Re-sample the displacement map at this new UV coordinate.
                                        current_sampled_displacement =
                                            sample_nearest(current_uv, map, None).0 as f32 / 255.0;

                                        // Update "current" layer depth for our next loop iteration.
                                        current_layer_depth += layer_depth;
                                    }

                                    // Interpolate between the sampled
                                    // displacements at the previous layer and
                                    // the current layer.

                                    let previous_uv = current_uv + uv_step;

                                    let after_depth =
                                        current_sampled_displacement - current_layer_depth;

                                    let before_depth =
                                        (sample_nearest(previous_uv, map, None).0 as f32 / 255.0)
                                            - current_layer_depth
                                            + layer_depth;

                                    let alpha = after_depth / (after_depth - before_depth);

                                    previous_uv * alpha + current_uv * (1.0 - alpha)
                                };

                            if displacement != 0.0 {
                                let fragment_to_view_direction_tangent_space =
                                    (out.tangent_space_info.view_position
                                        - out.tangent_space_info.fragment_position)
                                        .as_normal();

                                out.uv = get_parallax_mapped_uv(
                                    out.uv,
                                    fragment_to_view_direction_tangent_space,
                                    displacement,
                                );

                                if out.uv.x < 0.0
                                    || out.uv.x > 1.0
                                    || out.uv.y < 0.0
                                    || out.uv.y > 1.0
                                {
                                    return None;
                                }
                            }
                        }
                        Err(err) => {
                            panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                        }
                    }
                }
            }
            None => {
                panic!("Failed to get Material from MaterialCache: {}", name);
            }
        }
    }

    Some(out)
};

use crate::{
    color::Color,
    render::options::shader::RenderShaderOptions,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext,
        geometry::{sample::GeometrySample, GeometryShaderFn},
    },
    texture::sample::{sample_bilinear_u8, sample_nearest_u8},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

#[allow(non_upper_case_globals)]
pub static DefaultGeometryShader: GeometryShaderFn = |context: &ShaderContext,
                                                      resources: &SceneResources,
                                                      options: &RenderShaderOptions,
                                                      interpolant: &DefaultVertexOut|
 -> Option<GeometrySample> {
    let mut out = GeometrySample {
        uv: interpolant.uv,
        position_world_space: interpolant.position_world_space,
        position_view_space: interpolant.position_view_space,
        normal_world_space: interpolant.normal_world_space,
        tangent_space_info: interpolant.tangent_space_info,
        depth: interpolant.depth,
        roughness: 1.0,
        metallic: 0.0,
        albedo: interpolant.color,
        ambient_factor: 1.0,
        specular_color: vec3::ONES,
        specular_exponent: 8,
        emissive_color: Default::default(),
        alpha: 1.0,
    };

    if let Some(material_handle) = &context.active_material {
        if let Ok(entry) = resources.material.borrow().get(material_handle) {
            let material = &entry.item;

            // Displacement (height)
            if let (Some(displacement_map_handle), true) = (
                &material.displacement_map,
                options.displacement_mapping_active,
            ) {
                match resources.texture_u8.borrow().get(displacement_map_handle) {
                    Ok(entry) => {
                        let map = &entry.item;

                        let (r, _g, _b) = sample_nearest_u8(interpolant.uv, map, None);

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
                                        sample_nearest_u8(current_uv, map, None).0 as f32 / 255.0;

                                    // Update "current" layer depth for our next loop iteration.
                                    current_layer_depth += layer_depth;
                                }

                                // Interpolate between the sampled displacements
                                // at the previous layer and the current layer.

                                let previous_uv = current_uv + uv_step;

                                let after_depth =
                                    current_sampled_displacement - current_layer_depth;

                                let before_depth =
                                    (sample_nearest_u8(previous_uv, map, None).0 as f32 / 255.0)
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

                            if out.uv.x < 0.0 || out.uv.x > 1.0 || out.uv.y < 0.0 || out.uv.y > 1.0
                            {
                                return None;
                            }
                        }
                    }
                    Err(err) => {
                        panic!(
                            "Failed to get TextureMap from Arena: {:?}: {}",
                            displacement_map_handle, err
                        )
                    }
                }
            }

            // Surface normal mapping
            if let (Some(normal_map_handle), true) =
                (&material.normal_map, options.normal_mapping_active)
            {
                match resources.texture_u8.borrow().get(normal_map_handle) {
                    Ok(entry) => {
                        let map = &entry.item;

                        let (r, g, b) = sample_nearest_u8(out.uv, map, None);

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

                        out.normal_world_space = (tangent_space_normal
                            * interpolant.tangent_space_info.tbn)
                            .to_vec3()
                            .as_normal();

                        out.tangent_space_info.normal = tangent_space_normal.to_vec3().as_normal();
                    }
                    Err(err) => {
                        panic!(
                            "Failed to get TextureMap from Arena: {:?}: {}",
                            normal_map_handle, err
                        )
                    }
                }
            }

            // Ambient occlusion mapping
            if let (Some(ambient_occlusion_map_handle), true) = (
                &material.ambient_occlusion_map,
                options.ambient_occlusion_mapping_active,
            ) {
                match resources
                    .texture_u8
                    .borrow()
                    .get(ambient_occlusion_map_handle)
                {
                    Ok(entry) => {
                        let map = &entry.item;

                        let (r, _g, _b) = sample_nearest_u8(out.uv, map, None);

                        out.ambient_factor = r as f32 / 255.0;
                    }
                    Err(err) => {
                        panic!(
                            "Failed to get TextureMap from Arena: {:?}: {}",
                            ambient_occlusion_map_handle, err
                        )
                    }
                }
            }

            // Specular color
            match (
                &material.specular_color_map,
                options.specular_exponent_mapping_active,
            ) {
                (Some(specular_color_map_handle), true) => {
                    match &resources.texture_u8.borrow().get(specular_color_map_handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, g, b) = sample_nearest_u8(out.uv, map, None);

                            let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                            color.srgb_to_linear();

                            out.specular_color = color;
                        }
                        Err(_) => panic!("Invalid TextureMap handle!"),
                    }
                }
                _ => {
                    out.specular_color = material.specular_color;
                }
            }

            // Specular exponent
            match (
                &material.specular_exponent_map,
                options.specular_exponent_mapping_active,
            ) {
                (Some(specular_exponent_map_handle), true) => match &resources
                    .texture_u8
                    .borrow()
                    .get(specular_exponent_map_handle)
                {
                    Ok(entry) => {
                        let map = &entry.item;

                        let (r, _g, _b) = sample_nearest_u8(out.uv, map, None);

                        out.specular_exponent = r;
                    }
                    Err(_) => panic!("Invalid TextureMap handle!"),
                },
                _ => {
                    out.specular_exponent = material.specular_exponent;
                }
            }

            // Emissive color
            match material.emissive_color_map {
                Some(emissive_color_map_handle) => match resources
                    .texture_u8
                    .borrow()
                    .get(&emissive_color_map_handle)
                {
                    Ok(entry) => {
                        let map = &entry.item;

                        let (r, g, b) = sample_nearest_u8(out.uv, map, None);

                        let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                        color.srgb_to_linear();

                        out.emissive_color = color;
                    }
                    Err(err) => {
                        panic!(
                            "Failed to get TextureMap from Arena: {:?}: {}",
                            emissive_color_map_handle, err
                        )
                    }
                },
                None => {
                    out.emissive_color = material.emissive_color;
                }
            }

            // Alpha transparency
            match material.alpha_map {
                Some(alpha_map_handle) => {
                    match resources.texture_u8.borrow().get(&alpha_map_handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = sample_nearest_u8(out.uv, map, None);

                            out.alpha = r as f32 / 255.0;
                        }
                        Err(err) => {
                            panic!(
                                "Failed to get TextureMap from Arena: {:?}: {}",
                                alpha_map_handle, err
                            )
                        }
                    }
                }
                None => {
                    out.alpha = 1.0 - material.transparency;
                }
            }

            // Albedo color
            match (options.albedo_mapping_active, material.albedo_map) {
                (true, Some(albedo_map_handle)) => {
                    match resources.texture_u8.borrow().get(&albedo_map_handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, g, b) = if options.bilinear_active {
                                sample_bilinear_u8(out.uv, map, None)
                            } else {
                                sample_nearest_u8(out.uv, map, None)
                            };

                            let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                            color.srgb_to_linear();

                            out.albedo = color;
                        }
                        Err(err) => {
                            panic!(
                                "Failed to get TextureMap from Arena: {:?}: {}",
                                albedo_map_handle, err
                            )
                        }
                    }
                }
                _ => {
                    out.albedo = material.albedo;
                }
            }

            // Roughness
            match (material.roughness_map, options.roughness_mapping_active) {
                (Some(roughness_map_handle), true) => {
                    match resources.texture_u8.borrow().get(&roughness_map_handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = if options.bilinear_active {
                                sample_bilinear_u8(out.uv, map, None)
                            } else {
                                sample_nearest_u8(out.uv, map, None)
                            };

                            out.roughness = r as f32 / 255.0;
                        }
                        Err(err) => {
                            panic!(
                                "Failed to get TextureMap from Arena: {:?}: {}",
                                roughness_map_handle, err
                            )
                        }
                    }
                }
                _ => {
                    out.roughness = material.roughness;
                }
            }

            // Metallic
            match (material.metallic_map, options.metallic_mapping_active) {
                (Some(metallic_map_handle), true) => {
                    match resources.texture_u8.borrow().get(&metallic_map_handle) {
                        Ok(entry) => {
                            let map = &entry.item;

                            let (r, _g, _b) = if options.bilinear_active {
                                sample_bilinear_u8(out.uv, map, None)
                            } else {
                                sample_nearest_u8(out.uv, map, None)
                            };

                            out.metallic = r as f32 / 255.0;
                        }
                        Err(err) => {
                            panic!(
                                "Failed to get TextureMap from Arena: {:?}: {}",
                                metallic_map_handle, err
                            )
                        }
                    }
                }
                _ => {
                    out.metallic = material.metallic;
                }
            }

            // // Sheen
            // match material.sheen_map {
            //     Some(sheen_map_handle) => match resources.texture.borrow().get(&sheen_map_handle) {
            //         Ok(entry) => {
            //             let map = &entry.item;

            //             let (r, _g, _b) = sample_nearest_u8(out.uv, map, None);

            //             out.sheen = r as f32 / 255.0;
            //         }
            //         Err(err) => {
            //             panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
            //         }
            //     },
            //     None => {
            //         out.sheen = material.sheen;
            //     }
            // }

            // out.clearcoat_thickness = material.clearcoat_thickness;
            // out.clearcoat_roughness = material.clearcoat_roughness;
            // out.anisotropy = material.anisotropy;
            // out.anisotropy_rotation = material.anisotropy_rotation;
        }
    }

    Some(out)
};

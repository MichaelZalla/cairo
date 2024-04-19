use crate::{
    color::{self, Color},
    scene::light::PointLight,
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
                                                        options: &GeometryShaderOptions,
                                                        interpolant: &DefaultVertexOut|
 -> Option<GeometrySample> {
    let mut out: GeometrySample = Default::default();

    out.stencil = true;
    out.uv = interpolant.uv;
    out.normal = interpolant.normal.to_vec3();
    out.tangent_space_info = interpolant.tangent_space_info;
    out.world_pos = interpolant.world_pos;
    out.depth = interpolant.depth;

    // Displacement (height)

    match (options.displacement_mapping_active, &context.active_material) {
        (true, Some(name)) => {
            match &context.resources {
                Some(resources) => {
                    match resources.borrow().material.borrow().get(&name) {
                        Some(material) => {
                            match material.displacement_map {
                                Some(handle) => {
                                    match resources.borrow().texture.borrow().get(&handle) {
                                        Ok(entry) => {
                                            let map = &entry.item;

                                            let (r, _g, _b) = sample_nearest(interpolant.uv, &map, None);
            
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
            
                                                    // let layer_count = LAYER_COUNT_MAX;
            
                                                    let layer_depth: f32 = 1.0 / layer_count;
            
                                                    let p = Vec2 {
                                                        x: fragment_to_view_direction_tangent_space.x
                                                            / fragment_to_view_direction_tangent_space.z,
                                                        y: fragment_to_view_direction_tangent_space.y
                                                            / fragment_to_view_direction_tangent_space.z,
                                                        z: 1.0,
                                                    } * displacement
                                                        * *(&material.displacement_scale);
            
                                                    let uv_step = p / layer_count;
            
                                                    let mut current_layer_depth = 0.0;
                                                    let mut current_uv = uv;
                                                    let mut current_sampled_displacement = displacement;
            
                                                    while current_layer_depth < current_sampled_displacement {
                                                        // Take a step along P.
                                                        current_uv -= uv_step;
            
                                                        // Re-sample the displacement map at this new UV coordinate.
                                                        current_sampled_displacement =
                                                            sample_nearest(current_uv, &map, None).0 as f32
                                                                / 255.0;
            
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
                                                        (sample_nearest(previous_uv, &map, None).0 as f32
                                                            / 255.0)
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
                                            panic!(
                                                "Failed to get TextureMap from Arena: {:?}: {}",
                                                name, err
                                            )
                                        } 
                                    }
                                }
                                None => {
                                    // No displacement map defined on this material.
                                }
                            }
                        }
                        None => {
                            panic!("Failed to get Material from MaterialCache: {}", name);
                        }
                    }
                }
                None => {
                    // No resources bound to this shader context.
                }
            }
        }
        _ => {
            // No active material set for this shader context, or dispacement
            // mapping is disabled.
        }
    }

    // World-space surface normal

    match (options.normal_mapping_active, &context.active_material) {
        (true, Some(name)) => {
            match &context.resources {
                Some(resources) => {
                    match resources.borrow().material.borrow().get(&name) {
                        Some(material) => {
                            match material.normal_map {
                                Some(handle) => {
                                    match resources.borrow().texture.borrow().get(&handle) {
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

                                            out.normal = (tangent_space_normal
                                                * interpolant.tangent_space_info.tbn)
                                                .to_vec3()
                                                .as_normal();

                                            out.tangent_space_info.normal =
                                                tangent_space_normal.to_vec3().as_normal();
                                        }
                                        Err(err) => {
                                            panic!(
                                                "Failed to get TextureMap from Arena: {:?}: {}",
                                                name, err
                                            )
                                        }
                                    }
                                }
                                None => {
                                    // No normal map defined for this material.
                                }
                            }
                        }
                        None => {
                            panic!("Failed to get Material from MaterialCache: {}", name);
                        }
                    }
                }
                None => {
                    // No resources bound to this shader context.
                }
            }
        }
        _ => (),
    }

    // Ambient lighting (AO)

    match &context.active_material {
        Some(name) => {
            match &context.resources {
                Some(resources) => match resources.borrow().material.borrow().get(&name) {
                    Some(material) => match (
                        options.ambient_occlusion_mapping_active,
                        material.ambient_occlusion_map,
                    ) {
                        (true, Some(handle)) => match resources.borrow().texture.borrow().get(&handle) {
                            Ok(entry) => {
                                let map = &entry.item;

                                let (r, _g, _b) = sample_nearest(out.uv, map, None);

                                out.ambient_factor = r as f32 / 255.0;
                            }
                            Err(err) => {
                                panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                            }
                        },
                        _ => {
                            // No ambient occlusion map defined for this
                            // material, or ambient occlusion mapping is
                            // disabled.

                            out.ambient_factor = 1.0;
                        }
                    },
                    None => {
                        panic!("Failed to get Material from MaterialCache: {}", name)
                    }
                },
                None => {
                    // No resources bound to this shader context.

                    out.ambient_factor = 1.0;
                }
            }
        }
        None => {
            // No active material bound to this shader context.

            out.ambient_factor = 1.0;
        }
    }

    // Diffuse lighting

    match &context.active_material {
        Some(name) => {
            match &context.resources {
                Some(resources) => match resources.borrow().material.borrow().get(&name) {
                    Some(material) => {
                        match (options.diffuse_mapping_active, material.diffuse_map) {
                            (true, Some(handle)) => match resources.borrow().texture.borrow().get(&handle) {
                                Ok(entry) => {
                                    let map = &entry.item;

                                    let (r, g, b) = if options.bilinear_active {
                                        sample_bilinear(out.uv, map, None)
                                    } else {
                                        sample_nearest(out.uv, map, None)
                                    };

                                    out.diffuse = color::Color::rgb(r, g, b).to_vec3() / 255.0;
                                }
                                Err(err) => {
                                    panic!(
                                        "Failed to get TextureMap from Arena: {:?}: {}",
                                        name, err
                                    )
                                }
                            },
                            _ => {
                                // No diffuse map defined for this material, or
                                // diffuse mapping is disabled.

                                out.diffuse = material.diffuse_color;
                            }
                        }
                    }
                    None => {
                        panic!("Failed to get Material from MaterialCache: {}", name)
                    }
                },
                None => {
                    // No resources bound to this shader context.

                    out.diffuse = color::WHITE.to_vec3() / 255.0;
                }
            }
        }
        None => {
            // No active material bound to this shader context.

            out.diffuse = color::WHITE.to_vec3() / 255.0;
        }
    }

    // Specular lighting

    let default_point_light: Option<PointLight> = if context.point_lights.len() > 0 {
        let handle = context.point_lights[0];

        match &context.resources {
            Some(resources) => match resources.borrow().point_light.borrow().get(&handle) {
                Ok(entry) => {
                    let light = &entry.item;

                    Some(light.clone())
                }
                Err(err) => {
                    panic!("Failed to get PointLight from Arena: {:?}: {}", handle, err)
                }
            },
            None => None,
        }
    } else {
        None
    };

    match &context.active_material {
        Some(name) => {
            match &context.resources {
                Some(resources) => match resources.borrow().material.borrow().get(&name) {
                    Some(material) => {
                        out.specular_exponent = material.specular_exponent;

                        match (options.specular_mapping_active, &material.specular_map) {
                            (true, Some(handle)) => match &resources.borrow().texture.borrow().get(handle) {
                                Ok(entry) => {
                                    let map = &entry.item;

                                    let (r, g, b) = sample_nearest(out.uv, map, None);

                                    let r_f = r as f32;
                                    let g_f = g as f32;
                                    let b_f = b as f32;

                                    out.specular_intensity = (r_f + g_f + b_f) / 255.0;
                                }
                                Err(_) => panic!("Invalid TextureMap handle!"),
                            },
                            _ => {
                                // No specular map defined for this material, or
                                // specular mapping is disabled.

                                out.specular_intensity = if default_point_light.is_some() {
                                    default_point_light.unwrap().specular_intensity
                                } else {
                                    0.0
                                };
                            }
                        }
                    }
                    None => {
                        panic!("Failed to get Material from MaterialCache: {}", name)
                    }
                },
                None => {
                    // No resources bound to this shader context.

                    out.specular_exponent = 8;

                    out.specular_intensity = if default_point_light.is_some() {
                        default_point_light.unwrap().specular_intensity
                    } else {
                        0.0
                    };
                }
            }
        }
        None => {
            // No active material bound to this shader context.

            out.specular_exponent = 8;

            out.specular_intensity = if default_point_light.is_some() {
                default_point_light.unwrap().specular_intensity
            } else {
                0.0
            };
        }
    }

    // Emissive

    match &context.active_material {
        Some(name) => {
            match &context.resources {
                Some(resources) => match resources.borrow().material.borrow().get(&name) {
                    Some(material) => match material.emissive_map {
                        Some(handle) => match resources.borrow().texture.borrow().get(&handle) {
                            Ok(entry) => {
                                let map = &entry.item;

                                let (r, g, b) = sample_nearest(out.uv, map, None);

                                out.emissive = Color::rgb(r, g, b).to_vec3() / 255.0;
                            }
                            Err(err) => {
                                panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                            }
                        },
                        None => {
                            // No emissive map defined for this material.

                            out.emissive = material.emissive_color;
                        }
                    },
                    None => {
                        panic!("Failed to get Material from MaterialCache: {}", name)
                    }
                },
                None => {
                    // No resources bound to this shader context.

                    out.emissive = Default::default();
                }
            }
        }
        None => {
            // No active material bound to this shader context.

            out.emissive = Default::default();
        }
    }

    // Alpha (transparency)

    match &context.active_material {
        Some(name) => {
            match &context.resources {
                Some(resources) => match resources.borrow().material.borrow().get(&name) {
                    Some(material) => match material.alpha_map {
                        Some(handle) => match resources.borrow().texture.borrow().get(&handle) {
                            Ok(entry) => {
                                let map = &entry.item;

                                let (r, _g, _b) = sample_nearest(out.uv, &map, None);

                                out.alpha = r as f32 / 255.0;
                            }
                            Err(err) => {
                                panic!("Failed to get TextureMap from Arena: {:?}: {}", name, err)
                            }
                        },
                        None => {
                            // No alpha map defined for this material.

                            out.alpha = 1.0 - material.transparency;
                        }
                    },
                    None => {
                        panic!("Failed to get Material from MaterialCache: {}", name)
                    }
                },
                None => {
                    // No resources bound to this shader context.

                    out.alpha = 1.0;
                }
            }
        }
        None => {
            // No active material bound to this shader context.

            out.alpha = 1.0;
        }
    }

    Some(out)
};

use std::sync::RwLockReadGuard;

use crate::{
    color::{self, Color},
    scene::light::PointLight,
    shader::{
        geometry::{options::GeometryShaderOptions, sample::GeometrySample, GeometryShaderFn},
        ShaderContext,
    },
    texture::sample::{sample_bilinear, sample_nearest},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

pub static DEFAULT_GEOMETRY_SHADER: GeometryShaderFn = |context: &RwLockReadGuard<
    '_,
    ShaderContext,
>,
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

    match (options.displacement_mapping_active, context.active_material) {
        (true, Some(material_raw_mut)) => unsafe {
            let material = &(*material_raw_mut);

            match &material.displacement_map {
                Some(map) => {
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
                                    sample_nearest(current_uv, &map, None).0 as f32 / 255.0;

                                // Update "current" layer depth for our next loop iteration.
                                current_layer_depth += layer_depth;
                            }

                            // Interpolate between the sampled
                            // displacements at the previous layer and
                            // the current layer.

                            let previous_uv = current_uv + uv_step;

                            let after_depth = current_sampled_displacement - current_layer_depth;

                            let before_depth = (sample_nearest(previous_uv, &map, None).0 as f32
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

                        if out.uv.x < 0.0 || out.uv.x > 1.0 || out.uv.y < 0.0 || out.uv.y > 1.0 {
                            return None;
                        }
                    }
                }
                _ => (),
            }
        },
        _ => (),
    }

    // World-space surface normal

    match (options.normal_mapping_active, context.active_material) {
        (true, Some(material_raw_mut)) => {
            unsafe {
                match &(*material_raw_mut).normal_map {
                    Some(texture) => {
                        let (r, g, b) = sample_nearest(out.uv, texture, None);

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

                        out.normal = (tangent_space_normal * interpolant.tangent_space_info.tbn)
                            .to_vec3()
                            .as_normal();

                        out.tangent_space_info.normal = tangent_space_normal.to_vec3().as_normal();
                    }
                    None => (),
                }
            }
        }
        _ => (),
    }

    // Ambient lighting (AO)

    match (
        options.ambient_occlusion_mapping_active,
        context.active_material,
    ) {
        (true, Some(material_raw_mut)) => unsafe {
            match &(*material_raw_mut).ambient_occlusion_map {
                Some(map) => {
                    let (r, _g, _b) = sample_nearest(out.uv, map, None);
                    out.ambient_factor = r as f32 / 255.0;
                }
                _ => {
                    out.ambient_factor = 1.0;
                }
            }
        },
        _ => {
            out.ambient_factor = 1.0;
        }
    }

    // Diffuse lighting

    match context.active_material {
        Some(material_raw_mut) => unsafe {
            match (
                options.diffuse_mapping_active,
                &(*material_raw_mut).diffuse_map,
            ) {
                (true, Some(texture)) => {
                    let (r, g, b) = if options.bilinear_active {
                        sample_bilinear(out.uv, texture, None)
                    } else {
                        sample_nearest(out.uv, texture, None)
                    };

                    out.diffuse = color::Color::rgb(r, g, b).to_vec3() / 255.0;
                }
                _ => {
                    out.diffuse = (*material_raw_mut).diffuse_color;
                }
            }
        },
        None => {
            out.diffuse = color::WHITE.to_vec3() / 255.0;
        }
    }

    // Specular lighting

    let default_point_light: Option<PointLight> = if context.point_lights.len() > 0 {
        Some(context.point_lights[0])
    } else {
        None
    };

    match context.active_material {
        Some(material_raw_mut) => unsafe {
            out.specular_exponent = (*material_raw_mut).specular_exponent;

            match (
                options.specular_mapping_active,
                &(*material_raw_mut).specular_map,
            ) {
                (true, Some(map)) => {
                    let (r, g, b) = sample_nearest(out.uv, map, None);
                    let r_f = r as f32;
                    let g_f = g as f32;
                    let b_f = b as f32;
                    out.specular_intensity = (r_f + g_f + b_f) / 255.0;
                }
                _ => {
                    out.specular_intensity = if default_point_light.is_some() {
                        default_point_light.unwrap().specular_intensity
                    } else {
                        0.0
                    };
                }
            }
        },
        None => {
            out.specular_exponent = 8;

            out.specular_intensity = if default_point_light.is_some() {
                default_point_light.unwrap().specular_intensity
            } else {
                0.0
            };
        }
    }

    // Emissive

    match (options.emissive_mapping_active, context.active_material) {
        (true, Some(material_raw_mut)) => unsafe {
            match &(*material_raw_mut).emissive_map {
                Some(texture) => {
                    let (r, g, b) = sample_nearest(out.uv, texture, None);

                    out.emissive = Color::rgb(r, g, b).to_vec3() / 255.0;
                }
                None => out.emissive = (*material_raw_mut).emissive_color,
            }
        },
        _ => {
            out.emissive = Default::default();
        }
    }

    // Alpha (transparency)

    match context.active_material {
        Some(material_raw_mut) => unsafe {
            let material = &(*material_raw_mut);

            match &material.alpha_map {
                Some(map) => {
                    let (r, _g, _b) = sample_nearest(out.uv, &map, None);

                    out.alpha = r as f32 / 255.0;
                }
                None => {
                    out.alpha = 1.0 - material.transparency;
                }
            }
        },
        _ => {
            out.alpha = 1.0;
        }
    }

    Some(out)
};

use std::sync::RwLock;

use crate::{
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
    scene::light::PointLight,
    shader::{
        geometry::{sample::GeometrySample, GeometryShader, GeometryShaderOptions},
        ShaderContext,
    },
    texture::sample::{sample_bilinear, sample_nearest},
    vec::vec4::Vec4,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct DefaultGeometryShader<'a> {
    pub options: GeometryShaderOptions,
    context: &'a RwLock<ShaderContext>,
}

impl<'a> GeometryShader<'a> for DefaultGeometryShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<GeometryShaderOptions>) -> Self {
        match options {
            Some(options) => Self { context, options },
            None => Self {
                context,
                options: Default::default(),
            },
        }
    }

    fn get_options(&self) -> &GeometryShaderOptions {
        &self.options
    }

    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        self.options
            .update(keyboard_state, mouse_state, game_controller_state);
    }

    fn call(&self, interpolant: &DefaultVertexOut) -> GeometrySample {
        let context: std::sync::RwLockReadGuard<'_, ShaderContext> = self.context.read().unwrap();

        let mut out: GeometrySample = Default::default();

        out.stencil = true;
        out.uv = interpolant.uv;
        out.world_pos = interpolant.world_pos;
        out.depth = interpolant.depth;

        // World-space surface normal

        out.normal = interpolant.normal.to_vec3();

        out.tangent_space_info = interpolant.tangent_space_info;

        match (self.options.normal_mapping_active, context.active_material) {
            (true, Some(material_raw_mut)) => {
                unsafe {
                    match &(*material_raw_mut).normal_map {
                        Some(texture) => {
                            let (r, g, b) = sample_nearest(interpolant.uv, texture, None);

                            // Map the normal's components into the range [-1, 1].

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
                        None => (),
                    }
                }
            }
            _ => (),
        }

        // Ambient lighting (AO)

        match (
            self.options.ambient_occlusion_mapping_active,
            context.active_material,
        ) {
            (true, Some(material_raw_mut)) => unsafe {
                match &(*material_raw_mut).ambient_occlusion_map {
                    Some(map) => {
                        let (r, _g, _b) = sample_nearest(interpolant.uv, map, None);
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
                    self.options.diffuse_mapping_active,
                    &(*material_raw_mut).diffuse_map,
                ) {
                    (true, Some(texture)) => {
                        let (r, g, b) = if self.options.bilinear_active {
                            sample_bilinear(interpolant.uv, texture, None)
                        } else {
                            sample_nearest(interpolant.uv, texture, None)
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
                    self.options.specular_mapping_active,
                    &(*material_raw_mut).specular_map,
                ) {
                    (true, Some(map)) => {
                        let (r, g, b) = sample_nearest(interpolant.uv, map, None);
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

        match (
            self.options.emissive_mapping_active,
            context.active_material,
        ) {
            (true, Some(material_raw_mut)) => unsafe {
                match &(*material_raw_mut).emissive_map {
                    Some(texture) => {
                        let (r, g, b) = sample_nearest(interpolant.uv, texture, None);

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

        out
    }
}

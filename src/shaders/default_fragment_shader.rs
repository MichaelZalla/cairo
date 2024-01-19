use std::sync::RwLock;

use crate::{
    color::Color,
    device::{GameControllerState, KeyboardState, MouseState},
    shader::{
        fragment::{FragmentShader, FragmentShaderOptions},
        ShaderContext,
    },
    texture::sample::{sample_bilinear, sample_nearest},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct DefaultFragmentShader<'a> {
    pub options: FragmentShaderOptions,
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for DefaultFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<FragmentShaderOptions>) -> Self {
        match options {
            Some(options) => Self { context, options },
            None => Self {
                context,
                options: Default::default(),
            },
        }
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

    fn call(&self, out: &DefaultVertexOut) -> Color {
        let context: std::sync::RwLockReadGuard<'_, ShaderContext> = self.context.read().unwrap();

        // Calculate all lighting contributions

        let surface_normal = out.n.as_normal();

        let surface_normal_vec3 = Vec3 {
            x: surface_normal.x,
            y: surface_normal.y,
            z: surface_normal.z,
        };

        match (self.options.normal_mapping_active, context.active_material) {
            (true, Some(mat_raw_mut)) => {
                unsafe {
                    match &(*mat_raw_mut).normal_map {
                        Some(texture) => {
                            let (r, g, b) = sample_nearest(out.uv, texture, None);

                            let _map_normal = Vec4 {
                                x: (r as f32 / 255.0) * 2.0 - 1.0,
                                y: (g as f32 / 255.0) * 2.0 - 1.0,
                                z: (b as f32 / 255.0) * 2.0 - 1.0,
                                w: 1.0,
                            };

                            // @TODO Perturb the surface normal using the local
                            // tangent-space information read from `map`
                            //
                            // surface_normal = (surface_normal * out.TBN).as_normal();
                        }
                        None => (),
                    }
                }
            }
            _ => (),
        }

        // Calculate ambient light contribution

        let mut ambient_factor: f32 = 1.0;

        match (
            self.options.ambient_occlusion_mapping_active,
            context.active_material,
        ) {
            (true, Some(mat_raw_mut)) => unsafe {
                match &(*mat_raw_mut).ambient_occlusion_map {
                    Some(map) => {
                        let (r, _g, _b) = sample_nearest(out.uv, map, None);
                        ambient_factor = r as f32 / 255.0;
                    }
                    None => (),
                }
            },
            _ => (),
        }

        let ambient_contribution = context.ambient_light.contribute(ambient_factor);

        // Calculate directional light contribution

        let directional_light_contribution =
            context.directional_light.contribute(surface_normal_vec3);

        // Calculate point light contribution (including specular)

        let mut material_specular_exponent: Option<i32> = None;
        let mut material_specular_intensity: Option<f32> = None;

        match context.active_material {
            Some(mat_raw_mut) => unsafe {
                material_specular_exponent = Some((*mat_raw_mut).specular_exponent);

                match (
                    self.options.specular_mapping_active,
                    &(*mat_raw_mut).specular_map,
                ) {
                    (true, Some(map)) => {
                        let (r, g, b) = sample_nearest(out.uv, map, None);

                        let r_f = r as f32;
                        let g_f = g as f32;
                        let b_f = b as f32;

                        material_specular_intensity = Some((r_f + g_f + b_f) / 255.0);
                    }
                    _ => (),
                }
            },
            None => (),
        }

        // Calculate point light contributions.

        let mut point_light_contribution: Vec3 = Default::default();

        for point_light in &context.point_lights {
            let specular_exponent: i32 = match material_specular_exponent {
                Some(exponent) => exponent,
                None => context.default_specular_exponent,
            };

            let specular_intensity: f32 = match material_specular_intensity {
                Some(intensity) => intensity,
                None => point_light.specular_intensity,
            };

            point_light_contribution += point_light.contribute(
                out.world_pos,
                surface_normal_vec3,
                context.view_position,
                specular_intensity,
                specular_exponent,
            );
        }

        // Calculate spot light contributions.

        let mut spot_light_contribution: Vec3 = Default::default();

        for spot_light in &context.spot_lights {
            spot_light_contribution += spot_light.contribute(out.world_pos);
        }

        // Calculate emissive light contribution

        let mut emissive_light_contribution: Vec3 = Default::default();

        match (
            self.options.emissive_mapping_active,
            context.active_material,
        ) {
            (true, Some(mat_raw_mut)) => unsafe {
                match &(*mat_raw_mut).emissive_map {
                    Some(texture) => {
                        let (r, g, b) = sample_nearest(out.uv, texture, None);

                        emissive_light_contribution = Color::rgb(r, g, b).to_vec3() / 255.0;
                    }
                    None => emissive_light_contribution = (*mat_raw_mut).emissive_color,
                }
            },
            _ => (),
        }

        // Combine light intensities

        let total_contribution = ambient_contribution
            + directional_light_contribution
            + point_light_contribution
            + spot_light_contribution
            + emissive_light_contribution;

        // @TODO Honor each material's ambient, diffuse, and specular colors.

        let mut color: Vec3 = out.c;

        match context.active_material {
            Some(mat_raw_mut) => unsafe {
                match (
                    self.options.diffuse_mapping_active,
                    &(*mat_raw_mut).diffuse_map,
                ) {
                    (true, Some(texture)) => {
                        let (r, g, b) = if self.options.bilinear_active {
                            sample_bilinear(out.uv, texture, None)
                        } else {
                            sample_nearest(out.uv, texture, None)
                        };

                        color = Color::rgb(r, g, b).to_vec3() / 255.0;
                    }
                    _ => {
                        color = (*mat_raw_mut).diffuse_color;
                    }
                }
            },
            None => {}
        }

        color = Vec3 {
            x: color.x * color.x,
            y: color.y * color.y,
            z: color.z * color.z,
        };

        color = *((color * total_contribution).saturate());

        color = Vec3 {
            x: color.x.sqrt(),
            y: color.y.sqrt(),
            z: color.z.sqrt(),
        };

        return Color {
            r: (color.x * 255.0) as u8,
            g: (color.y * 255.0) as u8,
            b: (color.z * 255.0) as u8,
            a: 255 as u8,
        };
    }
}

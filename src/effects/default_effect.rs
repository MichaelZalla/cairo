use crate::{
    color::{self, Color},
    effect::Effect,
    material::Material,
    matrix::Mat4,
    scene::light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
    texture::sample::{sample_bilinear, sample_nearest},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub struct DefaultEffect {
    world_transform: Mat4,
    view_position: Vec4,
    view_inverse_transform: Mat4,
    world_view_transform: Mat4,
    projection_transform: Mat4,
    world_view_projection_transform: Mat4,
    ambient_light: AmbientLight,
    bilinear_active: bool,
    directional_light: DirectionalLight,
    point_light: PointLight,
    spot_light: SpotLight,
    default_specular_power: i32,
    active_material: Option<*const Material>,
}

impl DefaultEffect {
    pub fn new(
        world_transform: Mat4,
        view_position: Vec4,
        view_inverse_transform: Mat4,
        projection_transform: Mat4,
        ambient_light: AmbientLight,
        directional_light: DirectionalLight,
        point_light: PointLight,
        spot_light: SpotLight,
    ) -> Self {
        return DefaultEffect {
            world_transform,
            view_position,
            view_inverse_transform,
            projection_transform,
            world_view_transform: world_transform * view_inverse_transform,
            world_view_projection_transform: world_transform
                * view_inverse_transform
                * projection_transform,
            ambient_light,
            bilinear_active: false,
            directional_light,
            point_light,
            spot_light,
            default_specular_power: 8,
            active_material: None,
        };
    }

    pub fn set_camera_position(&mut self, position: Vec4) {
        self.view_position = position;
    }

    pub fn set_world_transform(&mut self, mat: Mat4) {
        self.world_transform = mat;

        self.world_view_transform = self.world_transform * self.view_inverse_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_view_inverse_transform(&mut self, mat: Mat4) {
        self.view_inverse_transform = mat;

        self.world_view_transform = self.world_transform * self.view_inverse_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_directional_light_direction(&mut self, direction: Vec4) {
        self.directional_light.direction = direction;
    }

    pub fn set_point_light_intensities(&mut self, intensities: Vec3) {
        self.point_light.intensities = intensities;
    }

    pub fn set_point_light_position(&mut self, position: Vec3) {
        self.point_light.position = position;
    }

    pub fn set_spot_light_intensities(&mut self, intensities: Vec3) {
        self.spot_light.intensities = intensities;
    }
}

impl Effect for DefaultEffect {
    type VertexIn = DefaultVertexIn;
    type VertexOut = DefaultVertexOut;

    fn get_projection(&self) -> Mat4 {
        return self.projection_transform;
    }

    fn set_projection(&mut self, projection_transform: Mat4) {
        self.projection_transform = projection_transform;
    }

    fn set_bilinear_active(&mut self, active: bool) {
        self.bilinear_active = active;

        println!(
            "Bilinear sampling {}.",
            if self.bilinear_active {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    fn set_active_material(&mut self, material_option: Option<*const Material>) {
        match material_option {
            Some(mat_raw_mut) => {
                self.active_material = Some(mat_raw_mut);
            }
            None => {
                self.active_material = None;
            }
        }
    }

    fn vs(&self, v: Self::VertexIn) -> Self::VertexOut {
        // Object-to-world-space vertex transform

        let mut out = Self::VertexOut::new();

        out.p = Vec4::new(v.p, 1.0) * self.world_view_projection_transform;

        let world_pos = Vec4::new(v.p, 1.0) * self.world_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        out.n = Vec4::new(v.n, 0.0) * self.world_transform;

        out.n = out.n.as_normal();

        out.c = v.c.clone();

        out.uv = v.uv.clone();

        return out;
    }

    fn ps(&self, interpolant: &<Self as Effect>::VertexOut) -> Option<Color> {
        let out = interpolant;

        // Check if this fragment can be discarded

        match self.active_material {
            Some(mat_raw_mut) => unsafe {
                match &(*mat_raw_mut).alpha_map {
                    Some(texture) => {
                        // Read in a per-fragment normal, with components in the range [0, 255].
                        let (r, _g, _b) = sample_nearest(out.uv, texture, None);

                        if r < 4 {
                            return None;
                        }
                    }
                    None => (),
                }
            },
            None => (),
        }

        // Calculate all lighting contributions

        let surface_normal = out.n.as_normal();

        let surface_normal_vec3 = Vec3 {
            x: surface_normal.x,
            y: surface_normal.y,
            z: surface_normal.z,
        };

        match self.active_material {
            Some(mat_raw_mut) => {
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
            None => (),
        }

        // Calculate ambient light contribution

        let mut ambient_factor: f32 = 1.0;

        match self.active_material {
            Some(mat_raw_mut) => unsafe {
                match &(*mat_raw_mut).ambient_occlusion_map {
                    Some(map) => {
                        let (r, _g, _b) = sample_nearest(out.uv, map, None);
                        ambient_factor = r as f32 / 255.0;
                    }
                    None => (),
                }
            },
            None => (),
        }

        let ambient_contribution = self.ambient_light.contribute(ambient_factor);

        // Calculate directional light contribution

        let directional_light_contribution = self.directional_light.contribute(surface_normal_vec3);

        // Calculate point light contribution (including specular)

        let specular_exponent: i32;
        let specular_intensity: f32;

        match self.active_material {
            Some(mat_raw_mut) => unsafe {
                specular_exponent = (*mat_raw_mut).specular_exponent;

                match &(*mat_raw_mut).specular_map {
                    Some(map) => {
                        let (r, g, b) = sample_nearest(out.uv, map, None);
                        let r_f = r as f32;
                        let g_f = g as f32;
                        let b_f = b as f32;
                        specular_intensity = (r_f + g_f + b_f) / 255.0;
                    }
                    None => {
                        specular_intensity = self.point_light.specular_intensity;
                    }
                }
            },
            None => {
                specular_exponent = self.default_specular_power;
                specular_intensity = self.point_light.specular_intensity;
            }
        }

        let point_light_contribution: Vec3 = self.point_light.contribute(
            out.world_pos,
            surface_normal_vec3,
            self.view_position,
            specular_intensity,
            specular_exponent,
        );

        // Calculate spot light contribution

        let spot_light_contribution = self.spot_light.contribute(out.world_pos);

        // Combine light intensities

        let total_contribution = ambient_contribution
            + directional_light_contribution
            + point_light_contribution
            + spot_light_contribution;

        // @TODO Honor each material's ambient, diffuse, and specular colors.

        let mut color: Vec3 = out.c;

        match self.active_material {
            Some(mat_raw_mut) => unsafe {
                match &(*mat_raw_mut).diffuse_map {
                    Some(texture) => {
                        let (r, g, b) = if self.bilinear_active {
                            sample_bilinear(out.uv, texture, None)
                        } else {
                            sample_nearest(out.uv, texture, None)
                        };

                        color = color::Color::rgb(r, g, b).to_vec3() / 255.0;
                    }
                    None => {
                        color = (*mat_raw_mut).diffuse_color;
                    }
                }
            },
            None => {}
        }

        color = *((color * total_contribution).saturate()) * 255.0;

        return Some(Color {
            r: color.x as u8,
            g: color.y as u8,
            b: color.z as u8,
            a: 255 as u8,
        });
    }
}

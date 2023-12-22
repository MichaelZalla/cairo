use crate::{
    color::{self, Color},
    effect::Effect,
    image::sample_from_uv,
    material::Material,
    matrix::Mat4,
    scene::light::{AmbientLight, DirectionalLight, PointLight},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub struct DefaultEffect {
    world_transform: Mat4,
    view_inverse_transform: Mat4,
    world_view_transform: Mat4,
    projection_transform: Mat4,
    world_view_projection_transform: Mat4,
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    point_light: PointLight,
    specular_intensity: f32,
    default_specular_power: i32,
    active_material: Option<*const Material>,
}

impl DefaultEffect {
    pub fn new(
        world_transform: Mat4,
        view_inverse_transform: Mat4,
        projection_transform: Mat4,
        ambient_light: AmbientLight,
        directional_light: DirectionalLight,
        point_light: PointLight,
    ) -> Self {
        return DefaultEffect {
            world_transform,
            view_inverse_transform,
            projection_transform,
            world_view_transform: world_transform * view_inverse_transform,
            world_view_projection_transform: world_transform
                * view_inverse_transform
                * projection_transform,
            ambient_light,
            directional_light,
            point_light,
            specular_intensity: 0.65,
            default_specular_power: 8,
            active_material: None,
        };
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

    pub fn set_point_light_position(&mut self, position: Vec3) {
        self.point_light.position = position;
    }
}

impl Effect for DefaultEffect {
    type VertexIn = DefaultVertexIn;
    type VertexOut = DefaultVertexOut;

    fn get_projection(&self) -> Mat4 {
        return self.projection_transform;
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

    fn ps(&self, interpolant: &<Self as Effect>::VertexOut) -> Color {
        let out = interpolant;

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
                            let (r, g, b) = sample_from_uv(out.uv, texture);

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

        // Ambient light contribution

        let ambient_contribution = self.ambient_light.intensities;

        // Calculate directional light intensity

        let directional_light_contribution = self.directional_light.intensities
            * (0.0 as f32).max((surface_normal_vec3 * -1.0).dot(Vec3 {
                x: self.directional_light.direction.x,
                y: self.directional_light.direction.y,
                z: self.directional_light.direction.z,
            }));

        // Calculate point light intensity

        let vertex_to_point_light = self.point_light.position - out.world_pos;

        let distance_to_point_light = vertex_to_point_light.mag();

        let direction_to_point_light = vertex_to_point_light / distance_to_point_light;

        let likeness = (0.0 as f32).max(surface_normal.dot(Vec4 {
            x: direction_to_point_light.x,
            y: direction_to_point_light.y,
            z: direction_to_point_light.z,
            w: 0.0,
        }));

        let mut point_light_contribution: Vec3 = Vec3::new();
        let mut specular_contribution: Vec3 = Vec3::new();

        if likeness > 0.0 {
            let attentuation = 1.0
                / (self.point_light.quadratic_attenuation * distance_to_point_light.powi(2)
                    + self.point_light.linear_attenuation * distance_to_point_light
                    + self.point_light.constant_attenuation);

            point_light_contribution =
                self.point_light.intensities * attentuation * (0.0 as f32).max(likeness);

            // Calculate specular light intensity

            let specular_exponent: i32;

            match self.active_material {
                Some(mat_raw_mut) => unsafe {
                    specular_exponent = (*mat_raw_mut).specular_exponent;
                },
                None => {
                    specular_exponent = self.default_specular_power;
                }
            }

            // point light projected onto surface normal
            let w = surface_normal_vec3 * vertex_to_point_light.dot(surface_normal_vec3);

            // vector to reflected light ray
            let r = w * 2.0 - vertex_to_point_light;

            // normal for reflected light
            let r_inverse_hat = r.as_normal() * -1.0;

            let similarity = (0.0 as f32).max(r_inverse_hat.dot(out.world_pos.as_normal()));

            specular_contribution = self.point_light.intensities
                * self.specular_intensity
                * similarity.powi(specular_exponent);
        }

        let total_contribution = ambient_contribution
            + directional_light_contribution
            + point_light_contribution
            + specular_contribution;

        // Calculate our color based on mesh color and light intensities

        let mut color: Vec3 = out.c;

        match self.active_material {
            Some(mat_raw_mut) => unsafe {
                match &(*mat_raw_mut).diffuse_map {
                    Some(texture) => {
                        let (r, g, b) = sample_from_uv(out.uv, texture);

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

        return Color {
            r: color.x as u8,
            g: color.y as u8,
            b: color.z as u8,
            a: 255 as u8,
        };
    }
}

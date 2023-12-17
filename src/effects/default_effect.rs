use crate::{
    color::{self, Color},
    context::ApplicationRenderingContext,
    effect::Effect,
    matrix::Mat4,
    scene::light::{AmbientLight, DirectionalLight, PointLight},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub struct DefaultEffect {
    world_view_transform: Mat4,
    projection_transform: Mat4,
    world_view_projection_transform: Mat4,
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    point_light: PointLight,
    specular_intensity: f32,
    specular_power: i32,
}

impl DefaultEffect {
    pub fn new(
        world_view_transform: Mat4,
        projection_transform: Mat4,
        ambient_light: AmbientLight,
        directional_light: DirectionalLight,
        point_light: PointLight,
        rendering_context: Option<&ApplicationRenderingContext>,
    ) -> Self {
        return DefaultEffect {
            world_view_transform,
            projection_transform,
            world_view_projection_transform: world_view_transform * projection_transform,
            ambient_light,
            directional_light,
            point_light,
            specular_intensity: 1.0,
            specular_power: 10,
        };
    }

    pub fn set_world_view_transform(&mut self, mat: Mat4) {
        self.world_view_transform = mat;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_projection_transform(&mut self, mat: Mat4) {
        self.projection_transform = mat;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_point_light_position(&mut self, position: Vec4) {
        self.point_light.position = position;
    }
}

impl Effect for DefaultEffect {
    type VertexIn = DefaultVertexIn;
    type VertexOut = DefaultVertexOut;

    fn get_projection(&self) -> Mat4 {
        return self.projection_transform;
    }

    fn vs(&self, v: Self::VertexIn) -> Self::VertexOut {
        let mut out = Self::VertexOut::new();

        out.p = Vec4::new(v.p, 1.0) * self.world_view_projection_transform;

        let world_pos = Vec4::new(v.p, 1.0) * self.world_view_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        out.n = Vec4::new(v.n, 0.0) * self.world_view_transform;

        out.n = out.n.as_normal();

        out.c = v.c.clone();

        out.uv = v.uv.clone();

        return out;
    }

    fn ps(&self, interpolant: &<Self as Effect>::VertexOut) -> Color {
        let out = interpolant;

        let surface_normal = out.n;

        let surface_normal_vec3 = Vec3 {
            x: surface_normal.x,
            y: surface_normal.y,
            z: surface_normal.z,
        };

        // Ambient light contribution

        let ambient_contribution = self.ambient_light.intensities;

        // Calculate directional light intensity

        let directional_light_direction_world_view =
            (self.directional_light.direction * self.world_view_transform).as_normal();

        let directional_light_contribution = self.directional_light.intensities
            * (0.0 as f32).max((surface_normal_vec3 * -1.0).dot(
                Vec3 {
                    x: directional_light_direction_world_view.x,
                    y: directional_light_direction_world_view.y,
                    z: directional_light_direction_world_view.z,
                },
                // Vec4::new(self.directional_light_direction, 1.0) * self.world_view_projection_transform
            ));

        // Calculate point light intensity

        let vertex_to_point_light = Vec3 {
            x: self.point_light.position.x,
            y: self.point_light.position.y,
            z: self.point_light.position.z,
        } - out.world_pos;

        let distance_to_point_light = vertex_to_point_light.mag();

        let normal_to_point_light = vertex_to_point_light / distance_to_point_light;

        let likeness = normal_to_point_light.dot(surface_normal_vec3 * -1.0);

        let attentuation = 1.0
            / (self.point_light.quadratic_attenuation * distance_to_point_light.powi(2)
                + self.point_light.linear_attenuation * distance_to_point_light
                + self.point_light.constant_attenuation);

        let mut point_light_contribution: Vec3 = Vec3::new();
        let mut specular_contribution: Vec3 = Vec3::new();

        if likeness < 0.0 {
            point_light_contribution = self.point_light.intensities
                * attentuation
                * (0.0 as f32).max(surface_normal_vec3.dot(normal_to_point_light));

            // Calculate specular light intensity

            // point light projected onto surface normal
            let w = surface_normal_vec3 * self.point_light.intensities.dot(surface_normal_vec3);

            // vector to reflected light ray
            let r = w * 2.0 - vertex_to_point_light;

            // normal for reflected light
            let r_inverse_hat = r.as_normal() * -1.0;

            specular_contribution = self.point_light.intensities
                * self.specular_intensity
                * (0.0 as f32)
                    .max(r_inverse_hat.dot(out.world_pos.as_normal()))
                    .powi(self.specular_power);
        }

        // Calculate our color based on mesh color and light intensities

        let mut color: Vec3;

        let checkerboard_subdivisions = 10.0;
        let checkerboard_subdivision_size = 1.0 / checkerboard_subdivisions;

        let is_dark_row =
            out.uv.y % (checkerboard_subdivision_size * 2.0) < checkerboard_subdivision_size;
        let should_flip =
            out.uv.x % (checkerboard_subdivision_size * 2.0) < checkerboard_subdivision_size;

        if (!is_dark_row && should_flip) || (is_dark_row && !should_flip) {
            color = color::BLACK.to_vec3();
        } else {
            color = color::WHITE.to_vec3();
        }

        color = *color
            .get_hadamard(
                ambient_contribution
                    + directional_light_contribution
                    + point_light_contribution
                    + specular_contribution,
            )
            .saturate()
            * 255.0;

        return Color {
            r: color.x as u8,
            g: color.y as u8,
            b: color.z as u8,
            a: 255 as u8,
        };
    }
}

use std::{f32::consts::PI, fmt};

use bitflags::bitflags;

use rigid_body_simulation_state::RigidBodySimulationState;

use crate::{
    color::{self, Color},
    geometry::primitives::aabb::AABB,
    matrix::Mat4,
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    transform::Transform3D,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::contact::{StaticContactKind, StaticContactList};

pub mod rigid_body_simulation_state;

#[derive(Debug, Copy, Clone)]
pub enum RigidBodyKind {
    Circle(f32),
    Sphere(f32),
}

impl Default for RigidBodyKind {
    fn default() -> Self {
        Self::Sphere(0.5)
    }
}

impl fmt::Display for RigidBodyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RigidBodyKind::Circle(radius) => format!("Circle({})", radius),
                RigidBodyKind::Sphere(radius) => format!("Sphere({})", radius),
            }
        )
    }
}

impl RigidBodyKind {
    pub fn get_moment_of_intertia(&self, mass: f32) -> (Mat4, Mat4) {
        match self {
            RigidBodyKind::Circle(radius) => {
                let scale = (PI * radius.powi(4)) / 2.0;

                let moment_of_inertia = Mat4::scale_uniform(scale);

                let inverse_moment_of_inertia = {
                    let inverse_scale = 1.0 / scale;

                    Mat4::scale_uniform(inverse_scale)
                };

                (moment_of_inertia, inverse_moment_of_inertia)
            }
            RigidBodyKind::Sphere(radius) => {
                let scale = (2.0 / 5.0) * mass * radius * radius;

                let moment_of_inertia = Mat4::scale_uniform(scale);

                let inverse_moment_of_inertia = {
                    let inverse_scale = 1.0 / scale;

                    Mat4::scale_uniform(inverse_scale)
                };

                (moment_of_inertia, inverse_moment_of_inertia)
            }
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct CollisionImpulse {
    pub contact_point: Vec3,
    pub contact_point_velocity: Vec3,
    pub normal: Vec3,
    pub normal_impulse: Vec3,
    pub tangent: Option<Vec3>,
    pub tangent_impulse: Option<Vec3>,
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct RigidBodyDebugFlags: u32 {
        const DRAW_VOLUME = 1;
        const DRAW_AABB = 1 << 1;
        const DRAW_LINEAR_VELOCITY = 1 << 2;
        const DRAW_ANGULAR_VELOCITY = 1 << 3;
        const DRAW_COLLISION_CONTACT_POINT = 1 << 4;
        const DRAW_COLLISION_CONTACT_POINT_VELOCITY = 1 << 5;
        const DRAW_COLLISION_NORMAL_IMPULSE = 1 << 6;
        const DRAW_COLLISION_TANGENT = 1 << 7;
        const DRAW_COLLISION_FRICTION_IMPULSE = 1 << 8;
        const DRAW_STATIC_CONTACT = 1 << 9;
    }
}

impl Default for RigidBodyDebugFlags {
    fn default() -> Self {
        RigidBodyDebugFlags::DRAW_VOLUME
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct RigidBody {
    pub kind: RigidBodyKind,
    pub transform: Transform3D,
    pub mass: f32,
    pub moment_of_inertia: Mat4,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
    pub color: Color,
    pub aabb: Option<AABB>,
    pub static_contacts: StaticContactList<6>,
    // Debug state
    pub collision_impulse: Option<CollisionImpulse>,
    pub debug_flags: RigidBodyDebugFlags,
    // Derived state
    inverse_mass: f32,
    inverse_moment_of_inertia: Mat4,
}

impl From<&RigidBody> for RigidBodySimulationState {
    fn from(body: &RigidBody) -> Self {
        RigidBodySimulationState {
            kind: body.kind,
            inverse_mass: body.inverse_mass,
            inverse_moment_of_inertia: body.inverse_moment_of_inertia,
            position: *body.transform.translation(),
            orientation: *body.transform.rotation(),
            linear_momentum: body.linear_momentum,
            angular_momentum: body.angular_momentum,
            static_contacts: body.static_contacts,
        }
    }
}

impl RigidBody {
    pub fn new(kind: RigidBodyKind, mass: f32, position: Vec3) -> Self {
        let inverse_mass = 1.0 / mass;

        let (moment_of_inertia, inverse_moment_of_inertia) = kind.get_moment_of_intertia(mass);

        let transform = {
            let mut transform = Transform3D::default();

            transform.set_translation(position);

            transform
        };

        let color = color::WHITE;

        let aabb = match kind {
            RigidBodyKind::Circle(_) => None,
            RigidBodyKind::Sphere(radius) => {
                let offset = vec3::ONES * radius;

                Some(AABB::from((position - offset, position + offset)))
            }
        };

        Self {
            kind,
            mass,
            inverse_mass,
            transform,
            moment_of_inertia,
            inverse_moment_of_inertia,
            color,
            aabb,
            ..Default::default()
        }
    }

    pub fn velocity(&self) -> Vec3 {
        self.linear_momentum * self.inverse_mass
    }

    fn inverse_moment_of_intertia_world_space(&self) -> Mat4 {
        let r = *self.transform.rotation().mat();

        r * self.inverse_moment_of_inertia * r.transposed()
    }

    pub fn angular_velocity(&self) -> Vec3 {
        let angular_momentum = Vec4::vector(self.angular_momentum);

        let inverse_moment_of_inertia_world_space = self.inverse_moment_of_intertia_world_space();

        (angular_momentum * inverse_moment_of_inertia_world_space).to_vec3()
    }

    pub fn apply_simulation_state(&mut self, state: &RigidBodySimulationState) {
        let (translation, mut orientation) = (state.position, state.orientation);

        self.transform.set_translation(translation);

        orientation.renormalize();

        self.transform.set_rotation(orientation);

        self.linear_momentum = state.linear_momentum;

        self.angular_momentum = state.angular_momentum;

        self.static_contacts = state.static_contacts;

        match self.kind {
            RigidBodyKind::Circle(_radius) => (),
            RigidBodyKind::Sphere(radius) => {
                if let Some(aabb) = &mut self.aabb {
                    aabb.min = state.position - vec3::ONES * radius;
                    aabb.max = state.position + vec3::ONES * radius;
                }
            }
        }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        let transform = &self.transform;

        let center = *transform.translation();

        if self.debug_flags.contains(RigidBodyDebugFlags::DRAW_VOLUME) {
            // Visualizes rigid body's geometric volume (sphere, circle, etc).

            match self.kind {
                RigidBodyKind::Sphere(radius) => {
                    let display_kind = EmptyDisplayKind::Sphere(12);

                    let transform_with_radius = Mat4::scale_uniform(radius) * *transform.mat();

                    renderer.render_empty(
                        &transform_with_radius,
                        display_kind,
                        true,
                        Some(self.color),
                    );
                }
                RigidBodyKind::Circle(_radius) => {
                    panic!();
                }
            };
        }

        if self.debug_flags.contains(RigidBodyDebugFlags::DRAW_AABB) {
            // Visualize the rigid body's AABB.

            if let Some(aabb) = &self.aabb {
                renderer.render_aabb(aabb, Default::default(), color::DARK_GRAY);
            }
        }

        if self
            .debug_flags
            .contains(RigidBodyDebugFlags::DRAW_LINEAR_VELOCITY)
        {
            // Visualizes the  rigid body's linear velocity.

            let linear_velocity = self.velocity();

            renderer.render_line(center, center + linear_velocity, color::LIGHT_GRAY);
        }

        if self
            .debug_flags
            .contains(RigidBodyDebugFlags::DRAW_ANGULAR_VELOCITY)
        {
            // Visualizes the rigid body's angular velocity.

            let angular_velocity = self.angular_velocity();

            renderer.render_line(center, center + angular_velocity, color::LIGHT_GRAY);
        }

        if let Some(impulse) = &self.collision_impulse {
            if self
                .debug_flags
                .contains(RigidBodyDebugFlags::DRAW_COLLISION_CONTACT_POINT)
            {
                // Visualizes the contact point.

                let scale = Mat4::scale_uniform(0.1);
                let translation = Mat4::translation(impulse.contact_point);
                let transform = scale * translation;

                renderer.render_empty(
                    &transform,
                    EmptyDisplayKind::Sphere(12),
                    false,
                    Some(color::LIGHT_GRAY),
                );
            }

            if self
                .debug_flags
                .contains(RigidBodyDebugFlags::DRAW_COLLISION_CONTACT_POINT_VELOCITY)
            {
                // Visualizes the contact point's velocity.

                renderer.render_line(
                    impulse.contact_point,
                    impulse.contact_point + impulse.contact_point_velocity,
                    color::LIGHT_GRAY,
                );
            }

            if self
                .debug_flags
                .contains(RigidBodyDebugFlags::DRAW_COLLISION_NORMAL_IMPULSE)
            {
                // Visualizes the collision response's normal impulse.

                renderer.render_line(
                    impulse.contact_point,
                    impulse.contact_point + impulse.normal_impulse,
                    color::BLUE,
                );
            }

            if self
                .debug_flags
                .contains(RigidBodyDebugFlags::DRAW_COLLISION_TANGENT)
            {
                // Visualizes the tangent vector chosen for the friction response.

                if let Some(tangent) = &impulse.tangent {
                    renderer.render_line(
                        impulse.contact_point,
                        impulse.contact_point + tangent,
                        color::GREEN,
                    );
                }
            }

            if self
                .debug_flags
                .contains(RigidBodyDebugFlags::DRAW_COLLISION_FRICTION_IMPULSE)
            {
                // Visualizes the collision response's friction impulse.

                if let Some(friction_impulse) = &impulse.tangent_impulse {
                    renderer.render_line(
                        impulse.contact_point,
                        impulse.contact_point + friction_impulse,
                        color::RED,
                    );
                }
            }
        }

        // Visualizes the rigid body's static contacts (if any).

        if self
            .debug_flags
            .contains(RigidBodyDebugFlags::DRAW_STATIC_CONTACT)
        {
            for contact in &self.static_contacts {
                let start = contact.point;

                // Contact normal
                renderer.render_line(
                    start,
                    start + contact.normal,
                    if let StaticContactKind::Resting = contact.kind {
                        color::WHITE
                    } else {
                        color::BLUE
                    },
                );

                // Contact point velocity
                renderer.render_line(start, start + contact.contact_point_velocity, color::ORANGE);

                // Contact point velocity (tangent)
                renderer.render_line(
                    start,
                    start + contact.tangent,
                    if let StaticContactKind::Resting = contact.kind {
                        color::WHITE
                    } else {
                        color::RED
                    },
                );

                // Contact point velocity (bitangent)
                renderer.render_line(
                    start,
                    start + contact.bitangent,
                    if let StaticContactKind::Resting = contact.kind {
                        color::WHITE
                    } else {
                        color::GREEN
                    },
                );
            }
        }
    }
}

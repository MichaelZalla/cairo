use cairo::{
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::CollisionSurface, contact::StaticContact,
            rigid_body::rigid_body_simulation_state::RigidBodySimulationState,
        },
    },
    vec::vec3::Vec3,
};

/// Trait for static colliders (plane, mesh, etc.) that can detect contacts and collisions.
pub trait StaticCollider {
    /// Detects a static contact (resting, sliding, or rolling) at the current body position.
    fn detect_static_contact(
        &self,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        radius: f32,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> Option<StaticContact>;

    /// Detects a collision between start and end positions, returning collision surface data.
    fn detect_collision(
        &self,
        start_position: Vec3,
        end_position: Vec3,
        radius: f32,
    ) -> Option<CollisionSurface>;

    /// Handles a complete collision for a single body, including:
    /// - Detecting collision and calculating impulse response
    /// - Applying collision impulses
    /// - Correcting penetration
    /// - Detecting and storing static contacts
    fn handle_collision(
        &self,
        h: f32,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        material: &PhysicsMaterial,
    );
}

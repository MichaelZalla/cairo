#[derive(Debug, Copy, Clone)]
pub struct PhysicsMaterial {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            static_friction: 0.0,
            dynamic_friction: 0.0,
            restitution: 1.0,
        }
    }
}

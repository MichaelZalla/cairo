#[derive(Debug, Copy, Clone)]
pub struct PhysicsMaterial {
    pub dynamic_friction: f32,
    pub restitution: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            dynamic_friction: 0.0,
            restitution: 1.0,
        }
    }
}

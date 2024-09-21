use cairo::vec::vec3::Vec3;

use crate::rigid_body_simulation_state::RigidBodySimulationState;

pub type Point = Vec3;
pub type Newtons = Vec3;

pub type Force = Box<dyn Fn(&RigidBodySimulationState, f32) -> (Newtons, Option<Point>)>;

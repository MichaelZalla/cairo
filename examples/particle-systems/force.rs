use cairo::vec::vec3::Vec3;

use crate::particle::Particle;

pub type Newtons = Vec3;

pub type Force = fn(&Particle) -> Newtons;

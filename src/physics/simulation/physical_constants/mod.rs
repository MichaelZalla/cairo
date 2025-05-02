use crate::vec::vec3::Vec3;

pub static EARTH_GRAVITY: f32 = 9.80665;

pub static EARTH_GRAVITY_ACCELERATION: Vec3 = Vec3 {
    x: 0.0,
    y: -EARTH_GRAVITY,
    z: 0.0,
};

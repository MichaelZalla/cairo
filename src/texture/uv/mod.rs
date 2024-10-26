use crate::vec::vec2::Vec2;

// Boundary UV coordinates

pub const TOP_LEFT: Vec2 = Vec2 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

pub const TOP_RIGHT: Vec2 = Vec2 {
    x: 1.0,
    y: 1.0,
    z: 0.0,
};

pub const BOTTOM_LEFT: Vec2 = Vec2 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

pub const BOTTOM_RIGHT: Vec2 = Vec2 {
    x: 1.0,
    y: 0.0,
    z: 0.0,
};

pub const CENTER: Vec2 = Vec2 {
    x: 0.5,
    y: 0.5,
    z: 0.0,
};

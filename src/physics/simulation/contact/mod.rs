use std::fmt;

use crate::vec::vec3::Vec3;

#[derive(Default, Debug, Copy, Clone)]
pub enum StaticContactKind {
    #[default]
    Resting,
    Sliding,
}

impl fmt::Display for StaticContactKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Resting => "Resting",
                Self::Sliding => "Sliding",
            }
        )
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticContact {
    pub kind: StaticContactKind,
    pub point: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

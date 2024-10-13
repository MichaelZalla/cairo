use std::{fmt::Display, str::FromStr};

#[derive(Default, Debug, Copy, Clone)]
pub enum FaceCullingWindingOrder {
    #[default]
    CounterClockwise = 0,
    Clockwise = 1,
}

impl Display for FaceCullingWindingOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::CounterClockwise => "CounterClockwise",
                Self::Clockwise => "Clockwise",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseFaceCullingWindingOrderErr {}

impl FromStr for FaceCullingWindingOrder {
    type Err = ParseFaceCullingWindingOrderErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CounterClockwise" => Ok(Self::CounterClockwise),
            "Clockwise" => Ok(Self::Clockwise),
            _ => Err(ParseFaceCullingWindingOrderErr {}),
        }
    }
}

pub static FACE_CULLING_WINDING_ORDER: [FaceCullingWindingOrder; 2] = [
    FaceCullingWindingOrder::CounterClockwise,
    FaceCullingWindingOrder::Clockwise,
];

#[derive(Default, Debug, Copy, Clone)]
pub enum FaceCullingReject {
    #[default]
    Backfaces = 0,
    Frontfaces = 1,
    None = 2,
}

impl Display for FaceCullingReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Backfaces => "Backfaces",
                Self::Frontfaces => "Frontfaces",
                Self::None => "None",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseFaceCullingRejectErr {}

impl FromStr for FaceCullingReject {
    type Err = ParseFaceCullingRejectErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Backfaces" => Ok(Self::Backfaces),
            "Frontfaces" => Ok(Self::Frontfaces),
            "None" => Ok(Self::None),
            _ => Err(ParseFaceCullingRejectErr {}),
        }
    }
}

pub static FACE_CULLING_REJECT: [FaceCullingReject; 3] = [
    FaceCullingReject::Backfaces,
    FaceCullingReject::Frontfaces,
    FaceCullingReject::None,
];

#[derive(Default, Debug, Copy, Clone)]
pub struct FaceCullingStrategy {
    pub reject: FaceCullingReject,
    pub winding_order: FaceCullingWindingOrder,
}

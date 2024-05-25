#[derive(Default, Debug, Copy, Clone)]
pub enum FaceCullingWindingOrder {
    Clockwise,
    #[default]
    CounterClockwise,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum FaceCullingReject {
    None,
    #[default]
    Backfaces,
    Frontfaces,
}
#[derive(Default, Debug, Copy, Clone)]
pub struct FaceCullingStrategy {
    pub reject: FaceCullingReject,
    pub winding_order: FaceCullingWindingOrder,
}

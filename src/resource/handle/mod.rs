use uuid::Uuid;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Handle {
    pub index: usize,
    pub uuid: Uuid,
}

impl Handle {}

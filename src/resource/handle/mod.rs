use uuid::Uuid;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Handle {
    pub index: usize,
    pub uuid: Uuid,
}

impl Handle {}

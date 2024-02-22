use uuid::Uuid;

#[derive(Default, Debug, Clone)]
pub struct Handle {
    pub index: usize,
    pub uuid: Uuid,
}

impl Handle {}

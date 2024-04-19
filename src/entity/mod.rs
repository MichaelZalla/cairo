use crate::resource::handle::Handle;

#[derive(Debug, Clone)]
pub struct Entity {
    pub mesh: Handle,
}

impl Entity {
    pub fn new(mesh: Handle) -> Self {
        Self { mesh }
    }
}

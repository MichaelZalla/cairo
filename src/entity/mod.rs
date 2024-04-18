use super::mesh::Mesh;

#[derive(Debug, Clone)]
pub struct Entity<'a> {
    pub mesh: &'a Mesh,
}

impl<'a> Entity<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        Self { mesh }
    }
}

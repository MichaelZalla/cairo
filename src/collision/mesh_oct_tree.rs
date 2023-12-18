use crate::{mesh::Mesh, vec::vec3::Vec3};

use super::{aabb::AABB, oct_tree::OctTreeNode};

#[derive(Default, Clone)]
pub struct MeshOctTree<'a> {
    mesh: &'a Mesh,
    pub tree: OctTreeNode<usize>,
}

impl<'a> MeshOctTree<'a> {
    pub fn new(mesh: &'a Mesh, level_capacity: usize, bounds: AABB) -> Self {
        let tree = OctTreeNode::<usize> {
            depth: 0,
            bounds,
            data_capacity: level_capacity,
            children: vec![],
            data: vec![],
        };

        let mut result = MeshOctTree { mesh, tree };

        for face_index in 0..result.mesh.faces.len() {
            result.insert_face(face_index);
        }

        return result;
    }

    pub fn insert_face(&mut self, face_index: usize) -> bool {
        let vertices = self.get_vertices_for_face(face_index);

        let aabb = AABB::new_from_triangle(&vertices.0, &vertices.1, &vertices.2);

        return self.tree.insert(face_index, &aabb);
    }

    fn get_vertices_for_face(&self, face_index: usize) -> (&Vec3, &Vec3, &Vec3) {
        let v0_index = self.mesh.faces[face_index].vertices.0;
        let v1_index = self.mesh.faces[face_index].vertices.1;
        let v2_index = self.mesh.faces[face_index].vertices.2;

        return (
            &self.mesh.vertices[v0_index],
            &self.mesh.vertices[v1_index],
            &self.mesh.vertices[v2_index],
        );
    }
}

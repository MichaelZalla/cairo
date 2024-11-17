use std::rc::Rc;

use crate::{
    geometry::primitives::aabb::AABB,
    mesh::{mesh_geometry::MeshGeometry, Mesh},
    vec::vec3::Vec3,
};

use super::oct_tree::OctTreeNode;

#[derive(Clone)]
pub struct MeshOctTree<'a> {
    geometry: &'a Rc<MeshGeometry>,
    mesh: &'a Mesh,
    pub root: OctTreeNode<usize>,
}

impl<'a> MeshOctTree<'a> {
    pub fn new(
        geometry: &'a Rc<MeshGeometry>,
        mesh: &'a Mesh,
        level_capacity: usize,
        bounds: AABB,
    ) -> Self {
        let root = OctTreeNode::<usize> {
            depth: 0,
            bounds,
            data_capacity: level_capacity,
            children: vec![],
            data: vec![],
        };

        let mut result = MeshOctTree {
            geometry,
            mesh,
            root,
        };

        for face_index in 0..result.mesh.faces.len() {
            result.insert_face(face_index);
        }

        result
    }

    pub fn insert_face(&mut self, face_index: usize) -> bool {
        let vertices = self.get_vertices_for_face(face_index);

        let aabb = AABB::new_from_triangle(vertices.0, vertices.1, vertices.2);

        self.root.insert(face_index, &aabb)
    }

    fn get_vertices_for_face(&self, face_index: usize) -> (&Vec3, &Vec3, &Vec3) {
        let v0_index = self.mesh.faces[face_index].vertices[0];
        let v1_index = self.mesh.faces[face_index].vertices[1];
        let v2_index = self.mesh.faces[face_index].vertices[2];

        (
            &self.geometry.vertices[v0_index],
            &self.geometry.vertices[v1_index],
            &self.geometry.vertices[v2_index],
        )
    }
}

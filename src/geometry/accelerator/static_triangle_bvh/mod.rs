use crate::{geometry::primitives::aabb::AABB, mesh::Mesh, vec::vec3::Vec3};

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticTriangle {
    vertices: [usize; 3],
    centroid: Vec3,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticTriangleBVHNode {
    pub aabb: AABB,
}

impl StaticTriangleBVH {
    pub fn new(mesh: &Mesh) -> Self {
        let root = StaticTriangleBVHNode { aabb: mesh.aabb };

        let vertices = &mesh.geometry.vertices;

        let num_faces = mesh.faces.len();

        let mut tris = vec![Default::default(); num_faces];

        for (face_index, face) in mesh.faces.iter().enumerate() {
            let (v0, v1, v2) = (
                vertices[face.vertices[0]],
                vertices[face.vertices[1]],
                vertices[face.vertices[2]],
            );

            tris[face_index] = StaticTriangle {
                vertices: face.vertices,
                centroid: (v0 + v1 + v2) * 0.33333,
            };
        }

        Self { tris, root }
    }
}

#[derive(Default, Debug, Clone)]
pub struct StaticTriangleBVH {
    pub tris: Vec<StaticTriangle>,
    pub root: StaticTriangleBVHNode,
}

use std::rc::Rc;

use crate::{
    geometry::primitives::aabb::AABB,
    mesh::{mesh_geometry::MeshGeometry, Mesh},
    vec::vec3::{self, Vec3A},
};

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticTriangle {
    pub vertices: [usize; 3],
    pub centroid: Vec3A,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticTriangleBVHNode {
    pub aabb: AABB,
    pub left_child_index: u32,
    pub primitives_start_index: u32,
    pub primitives_count: u32,
    pub depth: u8,
}

impl StaticTriangleBVHNode {
    pub fn is_leaf(&self) -> bool {
        self.primitives_count > 0
    }
}

#[derive(Debug, Clone)]
pub struct StaticTriangleBVH {
    pub geometry: Rc<MeshGeometry>,
    pub tris: Vec<StaticTriangle>,
    pub tri_indices: Vec<usize>,
    pub nodes: Vec<StaticTriangleBVHNode>,
    pub nodes_used: usize,
}

static BVH_NODE_LOAD_FACTOR: u32 = 4;

impl StaticTriangleBVH {
    pub fn new(mesh: &Mesh) -> Self {
        let num_tris = mesh.faces.len();

        let tri_indices = (0..num_tris).collect();

        let max_node_count = 2 * num_tris - 1;

        let mut nodes = vec![StaticTriangleBVHNode::default(); max_node_count];

        let tris = {
            let mut tris = vec![Default::default(); num_tris];

            let vertices = &mesh.geometry.vertices;

            for (face_index, face) in mesh.faces.iter().enumerate() {
                let (v0, v1, v2) = (
                    vertices[face.vertices[0]],
                    vertices[face.vertices[1]],
                    vertices[face.vertices[2]],
                );

                tris[face_index] = StaticTriangle {
                    vertices: face.vertices,
                    centroid: Vec3A {
                        v: (v0 + v1 + v2) * 0.33333,
                    },
                };
            }

            tris
        };

        let root_index = 0;

        let root = &mut nodes[root_index];

        root.primitives_start_index = 0;

        root.primitives_count = num_tris as u32;

        let mut bvh = Self {
            geometry: mesh.geometry.clone(),
            tris,
            tri_indices,
            nodes,
            nodes_used: 1,
        };

        bvh.recompute_node_aabb(root_index);

        bvh.subdivide(root_index);

        bvh
    }

    pub fn recompute_node_aabb(&mut self, node_index: usize) {
        let node = &mut self.nodes[node_index];

        let start = node.primitives_start_index as usize;

        let end = (node.primitives_start_index + node.primitives_count) as usize;

        let mut min = vec3::MAX;
        let mut max = vec3::MIN;

        for tri_index in &self.tri_indices[start..end] {
            let tri = &self.tris[*tri_index];

            let v0 = &self.geometry.vertices[tri.vertices[0]];
            let v1 = &self.geometry.vertices[tri.vertices[1]];
            let v2 = &self.geometry.vertices[tri.vertices[2]];

            min = min.min(v0);
            min = min.min(v1);
            min = min.min(v2);

            max = max.max(v0);
            max = max.max(v1);
            max = max.max(v2);
        }

        self.nodes[node_index].aabb = AABB::from_min_max(min, max);
    }

    fn subdivide(&mut self, split_node_index: usize) {
        // Base case.

        if self.nodes[split_node_index].primitives_count <= BVH_NODE_LOAD_FACTOR {
            return;
        }

        let root_aabb = &self.nodes[split_node_index].aabb;

        // Determine a split plane (axis, and position on that axis).

        let extent = Vec3A {
            v: root_aabb.extent(),
        };

        let split_axis = unsafe {
            let mut split_axis = 0;

            if extent.v.y > extent.v.x {
                split_axis = 1;
            }

            if extent.v.z > extent.a[split_axis] {
                split_axis = 2;
            }

            split_axis
        };

        let center = Vec3A {
            v: root_aabb.center(),
        };

        let split_position = unsafe { center.a[split_axis] };

        // Split the root's primitives into left and right bins.

        let start_index = self.nodes[split_node_index].primitives_start_index;

        let primitives_count = self.nodes[split_node_index].primitives_count;

        let mut i = start_index;

        let mut j = i + primitives_count - 1;

        unsafe {
            while i <= j {
                let tri_index = self.tri_indices[i as usize];

                let tri = &self.tris[tri_index];

                if tri.centroid.a[split_axis] < split_position {
                    i += 1;
                } else {
                    self.tri_indices.swap(i as usize, j as usize);
                    j -= 1;
                }
            }
        }

        let left_primitives_count = i - start_index;
        let right_primitives_count = primitives_count - left_primitives_count;

        if left_primitives_count == 0 || left_primitives_count == primitives_count {
            return;
        }

        let left_child_index = self.nodes_used;
        let right_child_index = left_child_index + 1;

        self.nodes[split_node_index].left_child_index = left_child_index as u32;

        self.nodes_used += 2;

        // Left child.

        self.nodes[left_child_index].depth = self.nodes[split_node_index].depth + 1;

        self.nodes[left_child_index].primitives_start_index =
            self.nodes[split_node_index].primitives_start_index;

        self.nodes[left_child_index].primitives_count = left_primitives_count;

        // Right child.

        self.nodes[right_child_index].depth = self.nodes[split_node_index].depth + 1;

        self.nodes[right_child_index].primitives_start_index = i;

        self.nodes[right_child_index].primitives_count = right_primitives_count;

        self.nodes[split_node_index].primitives_count = 0;

        // Update bounds.

        self.recompute_node_aabb(left_child_index);
        self.recompute_node_aabb(right_child_index);

        // Recurse.

        self.subdivide(left_child_index);
        self.subdivide(right_child_index);
    }
}

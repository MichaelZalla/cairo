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

    pub fn get_cost(&self) -> f32 {
        let extent = self.aabb.extent();

        self.primitives_count as f32 * extent.half_area_of_extent()
    }
}

#[derive(Default, Debug, Copy, Clone)]
struct Split {
    axis: usize,
    position: f32,
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
        let split = {
            let node = &self.nodes[split_node_index];

            // Base case.

            if node.primitives_count <= BVH_NODE_LOAD_FACTOR {
                return;
            }

            // Determine the split plane (axis) and position via some split strategy.

            let (split, split_cost) = {
                // self.split_strategy_midpoint(split_node_index)

                self.split_strategy_surface_area(split_node_index)
            };

            // Skip the subdivide if dividing the parent actually yields worse net cost.

            if split_cost >= node.get_cost() {
                return;
            }

            split
        };

        // Split the root's primitives into left and right bins.

        let start_index = self.nodes[split_node_index].primitives_start_index;

        let primitives_count = self.nodes[split_node_index].primitives_count;

        let mut i = start_index;

        let mut j = i + primitives_count - 1;

        unsafe {
            while i <= j {
                let tri_index = self.tri_indices[i as usize];

                let tri = &self.tris[tri_index];

                if tri.centroid.a[split.axis] < split.position {
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

    fn split_strategy_midpoint(&self, split_node_index: usize) -> Split {
        let split_node_aabb = &self.nodes[split_node_index].aabb;

        let extent = Vec3A {
            v: split_node_aabb.extent(),
        };

        let axis = unsafe {
            let mut split_axis = 0;

            if extent.v.y > extent.v.x {
                split_axis = 1;
            }

            if extent.v.z > extent.a[split_axis] {
                split_axis = 2;
            }

            split_axis
        };

        let position = unsafe {
            let center = Vec3A {
                v: split_node_aabb.center(),
            };

            center.a[axis]
        };

        Split { axis, position }
    }

    fn keep_best_split(
        &self,
        split_node_index: usize,
        split: Split,
        minimum_cost: &mut f32,
        best_axis: &mut isize,
        best_position: &mut f32,
    ) {
        let candidate_cost = self.get_split_cost_surface_area(split_node_index, split);

        if candidate_cost < *minimum_cost {
            *minimum_cost = candidate_cost;

            *best_axis = split.axis as isize;
            *best_position = split.position;
        }
    }

    fn split_strategy_surface_area(&self, split_node_index: usize) -> (Split, f32) {
        let mut best_axis: isize = -1;
        let mut best_position = 0_f32;

        let mut minimum_cost = f32::MAX;

        let split_node = &self.nodes[split_node_index];

        for axis in 0..3 {
            static BINNED_SPLITS: bool = true;

            if BINNED_SPLITS {
                let (extent_along_axis, min_position) = unsafe {
                    let (mut min_position, mut max_position) = (f32::MAX, f32::MIN);

                    for i in 0..split_node.primitives_count {
                        let tri_index_index = (split_node.primitives_start_index + i) as usize;

                        let tri_index = self.tri_indices[tri_index_index];

                        let tri = &self.tris[tri_index];

                        let centroid = &tri.centroid;

                        min_position = min_position.min(centroid.a[axis]);
                        max_position = max_position.max(centroid.a[axis]);
                    }

                    if min_position == max_position {
                        continue;
                    }

                    let extent_along_axis = max_position - min_position;

                    (extent_along_axis, min_position)
                };

                static NUM_INTERVALS: usize = 8;

                let interval_size = extent_along_axis / NUM_INTERVALS as f32;

                for i in 1..NUM_INTERVALS {
                    let position = min_position + interval_size * i as f32;

                    let candidate_split = Split { axis, position };

                    self.keep_best_split(
                        split_node_index,
                        candidate_split,
                        &mut minimum_cost,
                        &mut best_axis,
                        &mut best_position,
                    )
                }
            } else {
                for tri_index in (split_node.primitives_start_index as usize)
                    ..(split_node.primitives_start_index + split_node.primitives_count) as usize
                {
                    let tri = &self.tris[self.tri_indices[tri_index]];

                    let position = unsafe { tri.centroid.a[axis] };

                    let candidate_split = Split { axis, position };

                    self.keep_best_split(
                        split_node_index,
                        candidate_split,
                        &mut minimum_cost,
                        &mut best_axis,
                        &mut best_position,
                    )
                }
            }
        }

        if best_axis == -1 {
            panic!();
        }

        let split = Split {
            axis: best_axis as usize,
            position: best_position,
        };

        (split, minimum_cost)
    }

    fn get_split_cost_surface_area(&self, split_node_index: usize, split: Split) -> f32 {
        let (mut left_aabb, mut right_aabb) = (AABB::default(), AABB::default());

        let (mut left_count, mut right_count) = (0_usize, 0_usize);

        let split_node = &self.nodes[split_node_index];

        // Compute the left and right AABBs that would result from this split.

        for tri_index in (split_node.primitives_start_index as usize)
            ..(split_node.primitives_start_index + split_node.primitives_count) as usize
        {
            let tri = &self.tris[self.tri_indices[tri_index]];

            let (v0, v1, v2) =
                self.geometry
                    .get_vertices(tri.vertices[0], tri.vertices[1], tri.vertices[2]);

            unsafe {
                if tri.centroid.a[split.axis] < split.position {
                    left_count += 1;

                    left_aabb.grow(v0);
                    left_aabb.grow(v1);
                    left_aabb.grow(v2);
                } else {
                    right_count += 1;

                    right_aabb.grow(v0);
                    right_aabb.grow(v1);
                    right_aabb.grow(v2);
                }
            }
        }

        // Compute split cost.

        left_count as f32 * left_aabb.extent().half_area_of_extent()
            + right_count as f32 * right_aabb.extent().half_area_of_extent()
    }
}

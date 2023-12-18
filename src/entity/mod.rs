use super::{
    collision::{aabb::AABB, mesh_oct_tree::MeshOctTree},
    color,
    mesh::{primitive::make_box, Mesh},
    vec::vec3::Vec3,
};

#[derive(Default, Clone)]
pub struct Entity<'a> {
    pub position: Vec3,
    pub rotation: Vec3,
    pub mesh: &'a Mesh,
    pub bounds: AABB,
    pub oct_tree: MeshOctTree<'a>,
}

impl<'a> Entity<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        let bounds = Entity::make_bounding_box(&mesh);

        let width = bounds.right - bounds.left;
        let height = bounds.top - bounds.bottom;
        let depth = bounds.near - bounds.far;

        let collider_mesh_center = Vec3 {
            x: bounds.left + width / 2.0,
            y: bounds.bottom + height / 2.0,
            z: bounds.near - depth / 2.0,
        };

        let largest_dimension = width.max(height).max(depth);

        let half_dimension = largest_dimension / 2.0;

        let level_capacity = 64;

        let oct_tree_bounds = AABB::new(collider_mesh_center, half_dimension);

        let oct_tree = MeshOctTree::new(mesh, level_capacity, oct_tree_bounds);

        return Entity {
            position: Vec3::new(),
            rotation: Vec3::new(),
            mesh,
            bounds,
            oct_tree,
        };
    }

    fn make_bounding_box(mesh: &Mesh) -> AABB {
        let mut x_min: f32 = f32::MAX;
        let mut x_max: f32 = f32::MIN;

        let mut y_min: f32 = f32::MAX;
        let mut y_max: f32 = f32::MIN;

        let mut z_min: f32 = f32::MAX;
        let mut z_max: f32 = f32::MIN;

        for v in mesh.vertices.as_slice() {
            if v.x < x_min {
                x_min = v.x;
            } else if v.x > x_max {
                x_max = v.x;
            }

            if v.y < y_min {
                y_min = v.y;
            } else if v.y > y_max {
                y_max = v.y;
            }

            if v.z < z_min {
                z_min = v.z;
            } else if v.z > z_max {
                z_max = v.z;
            }
        }

        let width = x_max - x_min;
        let height = y_max - y_min;
        let depth = z_max - z_min;

        let result = AABB {
            center: Vec3 {
                x: x_min + width / 2.0,
                y: y_min + height / 2.0,
                z: z_min + depth / 2.0,
            },
            half_dimension: (x_max - x_min) / 2.0,
            left: x_min,
            right: x_max,
            top: y_max,
            bottom: y_min,
            near: z_max,
            far: z_min,
        };

        return result;
    }

    pub fn make_bounding_box_mesh(bounds: &AABB) -> Mesh {
        let width = bounds.right - bounds.left;
        let height = bounds.top - bounds.bottom;
        let depth = bounds.near - bounds.far;

        let mut bounding_box_mesh = make_box(width, height, depth);

        for v in bounding_box_mesh.vertices.as_mut_slice() {
            *v += bounds.center;
            // v.c = color::YELLOW.to_vec3() / 255.0;
        }

        return bounding_box_mesh;
    }
}

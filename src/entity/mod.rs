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
    pub collider_mesh: Mesh,
    pub oct_tree: MeshOctTree<'a>,
}

impl<'a> Entity<'a> {
    pub fn new(mesh: &'a Mesh) -> Self {
        let collider_mesh = Entity::make_collision_mesh(&mesh);

        let width = collider_mesh.vertices[1].x - collider_mesh.vertices[0].x;
        let height = collider_mesh.vertices[0].y - collider_mesh.vertices[2].y;
        let depth = collider_mesh.vertices[0].z - collider_mesh.vertices[4].z;

        let collider_mesh_center = Vec3 {
            x: collider_mesh.vertices[0].x + width / 2.0,
            y: collider_mesh.vertices[2].y + height / 2.0,
            z: collider_mesh.vertices[0].z - depth / 2.0,
        };

        let largest_dimension = width.max(height).max(depth);

        let half_dimension = largest_dimension / 2.0;

        let level_capacity = 64;

        let bounds = AABB::new(collider_mesh_center, half_dimension);

        let oct_tree = MeshOctTree::new(mesh, level_capacity, bounds);

        return Entity {
            position: Vec3::new(),
            rotation: Vec3::new(),
            mesh,
            collider_mesh,
            oct_tree,
        };
    }

    fn make_collision_mesh(mesh: &Mesh) -> Mesh {
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

        let mut collider = make_box(width, height, depth);

        let collider_offset = Vec3 {
            x: x_min + width / 2.0,
            y: y_min + height / 2.0,
            z: z_min + depth / 2.0,
        };

        for v in collider.vertices.as_mut_slice() {
            *v += collider_offset;
            // v.c = color::YELLOW.to_vec3() / 255.0;
        }

        return collider;
    }
}

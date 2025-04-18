use std::mem;

use crate::vec::{
    vec3::{self, Vec3, Vec3A},
    vec4::Vec4,
};

use super::{
    accelerator::static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
    primitives::{aabb::AABB, plane::Plane, ray::Ray},
};

pub fn test_aabb_aabb(a: &AABB, b: &AABB) -> bool {
    let a_min = Vec3A { v: a.min };
    let a_max = Vec3A { v: a.max };
    let b_min = Vec3A { v: b.min };
    let b_max = Vec3A { v: b.max };

    for axis in 0..3 {
        unsafe {
            if a_max.a[axis] < b_min.a[axis] || a_min.a[axis] > b_max.a[axis] {
                return false;
            }
        }
    }

    true
}

pub fn intersect_line_segment_plane(plane: &Plane, a: Vec3, b: Vec3) -> Option<(f32, Vec3)> {
    // Compute a t-value for the directed line intersecting the plane.

    let ab = b - a;

    let nominator = plane.d - plane.normal.dot(a);

    let denominator = plane.normal.dot(ab);

    if denominator == f32::EPSILON {
        // Line segment is parallel to the plane.

        return None;
    }

    let t = nominator / denominator;

    if (0.0..=1.0).contains(&t) {
        // If t lies in the range [0..1], compute the segment's intersection point.

        let q = a + ab * t;

        Some((t, q))
    } else {
        // Else, no intersection has occurred.

        None
    }
}

pub fn intersect_ray_triangle(
    ray: &mut Ray,
    bvh_instance_index: usize,
    triangle_index: usize,
    v0: &Vec3,
    v1: &Vec3,
    v2: &Vec3,
) {
    // See: https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;

    let ray_cross_edge2 = ray.direction.cross(edge2);

    let determinant = edge1.dot(ray_cross_edge2);

    if determinant > -f32::EPSILON && determinant < f32::EPSILON {
        // Ray is parallel to this triangle.

        return;
    }

    let determinant_inverse = 1.0 / determinant;

    let s = ray.origin - v0;

    let u = determinant_inverse * s.dot(ray_cross_edge2);

    if !(0.0..=1.0).contains(&u) {
        return;
    }

    let s_cross_edge1 = s.cross(edge1);

    let v = determinant_inverse * ray.direction.dot(s_cross_edge1);

    if v < 0.0 || (u + v) > 1.0 {
        return;
    }

    // The line that the ray follows intersects this triangle.

    let t = determinant_inverse * edge2.dot(s_cross_edge1);

    if t > f32::EPSILON && t < ray.t {
        // Closest intersection to this ray so far.

        ray.t = t;

        ray.colliding_bvh_index.replace(bvh_instance_index);

        ray.colliding_primitive.replace(triangle_index);
    }
}

pub fn test_ray_aabb(ray: &Ray, aabb: &AABB) -> f32 {
    let min = &aabb.min;
    let max = &aabb.max;

    let t_x1 = (min.x - ray.origin.x) * ray.one_over_direction.x;
    let t_x2 = (max.x - ray.origin.x) * ray.one_over_direction.x;

    let mut t_min = t_x1.min(t_x2);
    let mut t_max = t_x1.max(t_x2);

    let t_y1 = (min.y - ray.origin.y) * ray.one_over_direction.y;
    let t_y2 = (max.y - ray.origin.y) * ray.one_over_direction.y;

    t_min = t_min.max(t_y1.min(t_y2));
    t_max = t_max.min(t_y1.max(t_y2));

    let t_z1 = (min.z - ray.origin.z) * ray.one_over_direction.z;
    let t_z2 = (max.z - ray.origin.z) * ray.one_over_direction.z;

    t_min = t_min.max(t_z1.min(t_z2));
    t_max = t_max.min(t_z1.max(t_z2));

    if t_max >= t_min && t_min < ray.t && t_max > 0.0 {
        t_min
    } else {
        f32::MAX
    }
}

pub fn intersect_ray_aabb(ray: &mut Ray, node_index: usize, aabb: &AABB) {
    let mut t_min = 0.0_f32;
    let mut t_max = f32::MAX;

    let p = Vec3A::from_vec3(ray.origin);
    let d = Vec3A::from_vec3(ray.direction);

    let aabb_min = Vec3A::from_vec3(aabb.min);
    let aabb_max = Vec3A::from_vec3(aabb.max);

    let one_over_direction = Vec3A::from_vec3(ray.one_over_direction);

    for i in 0..3 {
        unsafe {
            if d.a[i].abs() < f32::EPSILON {
                // Ray is parallel to this slab.
                // If the ray origin is not within the slab, then no hit.

                if p.a[i] < aabb_min.a[i] || p.a[i] > aabb_max.a[i] {
                    return;
                }
            } else {
                // Compute the ray's intersection (t) value with near and far plane of this slab.

                let mut t1 = (aabb_min.a[i] - p.a[i]) * one_over_direction.a[i];
                let mut t2 = (aabb_max.a[i] - p.a[i]) * one_over_direction.a[i];

                // t1 as intersection of near plane, t2 as intersection of far plane.

                if t1 > t2 {
                    mem::swap(&mut t1, &mut t2);
                }

                // Compute the intersection of slab-intersection intervals.

                t_min = t_min.max(t1);
                t_max = t_max.min(t2);

                // Exit with no hit as soon as the slab intersection is empty.

                if t_min > t_max {
                    return;
                }
            }
        }
    }

    ray.colliding_primitive.replace(node_index);

    ray.t = ray.t.min(t_min);
}

pub fn intersect_ray_bvh(
    ray: &mut Ray,
    bvh_instance_index: usize,
    bvh_instance: &StaticTriangleBVHInstance,
) {
    let mut transformed_ray = *ray;

    transformed_ray.origin =
        (Vec4::new(transformed_ray.origin, 1.0) * bvh_instance.inverse_transform).to_vec3();

    transformed_ray.direction *= bvh_instance.inverse_transform;

    transformed_ray.one_over_direction = vec3::ONES / ray.direction;

    let original_t = transformed_ray.t;

    intersect_ray_bvh_node_sorted(&mut transformed_ray, bvh_instance_index, &bvh_instance.bvh);

    if transformed_ray.t < original_t {
        ray.t = transformed_ray.t;

        ray.colliding_bvh_index = transformed_ray.colliding_bvh_index;

        ray.colliding_primitive = transformed_ray.colliding_primitive;
    }
}

fn intersect_ray_bvh_node(
    ray: &mut Ray,
    bvh_instance_index: usize,
    bvh: &StaticTriangleBVH,
    node_index: usize,
) {
    let node = &bvh.nodes[node_index];

    if test_ray_aabb(ray, &node.aabb) == f32::MAX {
        return;
    };

    if node.is_leaf() {
        let start = node.primitives_start_index as usize;
        let end = start + node.primitives_count as usize;

        for tri_index_index in start..end {
            let tri_index = bvh.tri_indices[tri_index_index];

            let tri = &bvh.tris[tri_index];

            let [v0, v1, v2] = tri.vertices;

            let (v0, v1, v2) = bvh.geometry.get_vertices(v0, v1, v2);

            intersect_ray_triangle(ray, bvh_instance_index, tri_index, v0, v1, v2);
        }
    } else {
        intersect_ray_bvh_node(ray, bvh_instance_index, bvh, node.left_child_index as usize);

        intersect_ray_bvh_node(
            ray,
            bvh_instance_index,
            bvh,
            node.left_child_index as usize + 1,
        );
    }
}

pub fn intersect_ray_bvh_node_sorted(
    ray: &mut Ray,
    bvh_instance_index: usize,
    bvh: &StaticTriangleBVH,
) {
    let mut node = &bvh.nodes[0];

    let mut stack = vec![0_usize; 64];
    let mut stack_ptr = 0_usize;

    loop {
        if node.is_leaf() {
            let start = node.primitives_start_index as usize;
            let end = start + node.primitives_count as usize;

            for tri_index_index in start..end {
                let tri_index = bvh.tri_indices[tri_index_index];

                let tri = &bvh.tris[tri_index];

                let [v0, v1, v2] = tri.vertices;

                let (v0, v1, v2) = bvh.geometry.get_vertices(v0, v1, v2);

                intersect_ray_triangle(ray, bvh_instance_index, tri_index, v0, v1, v2);
            }

            if stack_ptr == 0 {
                break;
            } else {
                stack_ptr -= 1;

                let node_index = stack[stack_ptr];

                node = &bvh.nodes[node_index];
            }

            continue;
        }

        let mut near_child_index = node.left_child_index as usize;
        let mut far_child_index = near_child_index + 1;

        let mut near_distance = test_ray_aabb(ray, &bvh.nodes[near_child_index].aabb);
        let mut far_distance = test_ray_aabb(ray, &bvh.nodes[far_child_index].aabb);

        if near_distance > far_distance {
            mem::swap(&mut near_child_index, &mut far_child_index);
            mem::swap(&mut near_distance, &mut far_distance);
        }

        if near_distance == f32::MAX {
            if stack_ptr == 0 {
                break;
            } else {
                stack_ptr -= 1;

                let node_index = stack[stack_ptr];

                node = &bvh.nodes[node_index];
            }
        } else {
            node = &bvh.nodes[near_child_index];

            if far_distance != f32::MAX {
                stack[stack_ptr] = far_child_index;
                stack_ptr += 1;
            }
        }
    }
}

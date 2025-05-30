use std::mem;

use crate::{
    animation::lerp,
    vec::{
        vec3::{self, Vec3, Vec3A},
        vec4::Vec4,
    },
};

use super::{
    accelerator::static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
    primitives::{
        aabb::AABB, line_segment::LineSegment, plane::Plane, ray::Ray, sphere::Sphere,
        triangle::Triangle,
    },
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

pub fn intersect_line_segment_triangle(
    segment: &mut LineSegment,
    triangle: &Triangle,
) -> Option<Vec3> {
    let (p, q) = (segment.start, segment.end);

    // Compute the distance of P to the triangle's normal-facing plane.

    let p_distance = triangle.plane.get_signed_distance(&p);

    // Exit if start point P is behind the plane.

    if p_distance < 0.0 {
        return None;
    }

    // Compute the distance of Q to the triangle's normal-facing plane.

    let q_distance = triangle.plane.get_signed_distance(&q);

    // Exit if end point Q is in front of the plane.

    if q_distance >= 0.0 {
        return None;
    }

    // Compute the point-of-intersection S of the line PQ with the plane.

    let total_distance = p_distance - q_distance;

    let t = p_distance / total_distance;

    let s = lerp(p, q, t);

    // Compute the barycentric coordinate U; exit if outside the [0..1] range.

    let u = s.dot(triangle.edge_plane_bc.normal) - triangle.edge_plane_bc.d;

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    // Compute the barycentric coordinate V; exit if negative.

    let v = s.dot(triangle.edge_plane_ca.normal) - triangle.edge_plane_ca.d;

    if v < 0.0 {
        return None;
    }

    // Compute the barycentric coordinate W; exit if negative.

    let w = 1.0 - u - v;

    if w < 0.0 {
        return None;
    }

    // Segment PQ intersects triangle.

    if t < segment.t {
        segment.t = t;

        Some(Vec3 { x: u, y: v, z: w })
    } else {
        None
    }
}

pub fn test_sphere_sphere(a: Sphere, b: Sphere) -> bool {
    let ab = b.center - a.center;

    let r1_r2 = a.radius + b.radius;

    ab.mag_squared() <= r1_r2 * r1_r2
}

pub fn test_moving_spheres(a: Sphere, b: Sphere, v: Vec3, v_distance: f32) -> bool {
    intersect_moving_spheres(a, b, v, v_distance).is_some()
}

pub fn intersect_moving_spheres(
    a: Sphere,
    mut b: Sphere,
    v: Vec3,
    v_distance: f32,
) -> Option<(f32, Vec3)> {
    // Expands the radius of sphere B by that of sphere A.

    b.radius += a.radius;

    let ray = Ray::new(a.center, v.as_normal());

    match intersect_ray_sphere(&ray, &b) {
        Some((t, contact_point)) => {
            if t <= v_distance {
                Some((t, contact_point))
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn intersect_capsule_plane(
    c: Vec3,
    d: Vec3,
    radius: f32,
    plane: &Plane,
) -> Option<(f32, Vec3)> {
    let n = plane.normal;

    let signed_distance = n.dot(c) - plane.d;

    let mut t = 0.0;

    // Plane equation, displaced by r:
    //
    //   (n•X) = d +/-r
    //
    // Replaces X with the intersection point equation S(t) = C+t*v:
    //
    //   (n•(C+t*v)) = d +/-r
    //
    // Expanding the dot product:
    //
    //   (n•C) + t*(n•v) = d +/-r
    //
    // Solves for t:
    //
    //   t*(n•v) = -(n•C) + d +/-r
    //   t = (-(n•C) + d +/-r)/(n•v)
    //   t = (+/-r + -(n•C - d))/(n•v)
    //   t = (+/-r - (n•C - d))/(n•v)
    //   t = (+/-r - distance)/(n•v)

    if signed_distance.abs() <= radius {
        // The sphere is already overlapping the plane; set the
        // time-of-intersection to zero, and the point-of-intersection to the
        // deepest point on the sphere.

        Some((t, c - plane.normal * radius))
    } else {
        // Checks if a collision occurred between start and end positions.

        let v = d - c;

        let n_dot_v = n.dot(v);

        if n_dot_v * signed_distance >= 0.0 {
            // Sphere is moving parallel or away from the plane, so no
            // collision.

            None
        } else {
            // Sphere is moving towards the plane.

            // Chooses a plane offset, based on which side of the plane the
            // sphere is initially positioned at.
            let r = if signed_distance > 0.0 {
                radius
            } else {
                -radius
            };

            t = (r - signed_distance) / n_dot_v;

            // Computes the point on the line segment (CD) at which the
            // collision occurs.

            let segment_point = c + v * t;

            // Offsets the segment intersection point, towards the plane, by `radius` units.

            Some((t, segment_point - n * r))
        }
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

pub fn intersect_ray_sphere(ray: &Ray, sphere: &Sphere) -> Option<(f32, Vec3)> {
    let sphere_to_ray = ray.origin - sphere.center;

    // Ray originates outside of sphere when c > 0.

    let c = sphere_to_ray.mag_squared() - sphere.radius * sphere.radius;

    // Ray points away from sphere's center when b > 0.

    let b = sphere_to_ray.dot(ray.direction);

    // Exit if ray origin is outside of sphere and ray points away from sphere.

    if c > 0.0 && b > 0.0 {
        return None;
    }

    // Quadratic discriminant.

    let discriminant = b * b - c;

    // When discriminant is negative, the ray misses the sphere.

    if discriminant < 0.0 {
        return None;
    }

    // Ray must intersect sphere; find the smallest t-value for the ray.

    let mut t = -b - discriminant.sqrt();

    // A negative t-value indicates that the intersecting ray starts inside of the sphere.

    t = t.max(0.0);

    Some((t, ray.origin + ray.direction * t))
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

    let p = Vec3A::from(ray.origin);
    let d = Vec3A::from(ray.direction);

    let aabb_min = Vec3A::from(aabb.min);
    let aabb_max = Vec3A::from(aabb.max);

    let one_over_direction = Vec3A::from(ray.one_over_direction);

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
        (Vec4::position(transformed_ray.origin) * bvh_instance.inverse_transform).to_vec3();

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

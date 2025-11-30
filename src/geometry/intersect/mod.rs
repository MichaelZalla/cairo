pub mod medial_primitive;

use std::mem;

use crate::{
    animation::lerp,
    geometry::accelerator::static_triangle_bvh::StaticBVHCollision,
    vec::{
        vec3::{self, Vec3, Vec3A},
        vec4::Vec4,
    },
};

use super::{
    accelerator::static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
    primitives::{
        aabb::AABB,
        capsule::Capsule,
        line_segment::LineSegment,
        plane::Plane,
        ray::Ray,
        sphere::Sphere,
        triangle::{Triangle, TriangleIntersection, point_projects_onto_triangle},
    },
};

/// Classification of where a primitive intersects a triangle.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TriangleIntersectionKind {
    Face,
    Edge,
    Vertex,
}

#[derive(Debug, Copy, Clone)]
pub struct RaySphereIntersection {
    /// Distance along the ray direction where intersection occurs.
    /// This is the parametric t value: intersection_point = ray.origin + ray.direction * distance
    pub distance: f32,

    /// The point in world space where the intersection occurs.
    pub intersection_point: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct ContinuousCollision {
    /// Distance along the velocity direction vector where collision occurs.
    /// This is NOT a normalized [0,1] parameter - it's in the same units as the velocity vector.
    /// To convert to a time parameter, divide by velocity magnitude: t_seconds = distance / velocity_magnitude
    /// To convert to a timestep fraction, divide by velocity magnitude AND timestep: t_fraction = distance / (velocity_magnitude * timestep)
    pub distance: f32,

    /// The point in world space where the collision occurs.
    pub contact_point: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct CapsulePlaneIntersection {
    /// Parametric t value [0,1] along the capsule's movement from start to end position.
    /// 0.0 = collision at start position, 1.0 = collision at end position
    pub t: f32,

    /// The point on the capsule's surface where it contacts the plane.
    pub contact_point: Vec3,
}

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
) -> Option<TriangleIntersection> {
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

    // Check if intersection point is inside the triangle
    let barycentric = point_projects_onto_triangle(s, triangle)?;

    // Segment PQ intersects triangle.
    Some(TriangleIntersection { t, barycentric })
}

pub fn intersect_capsule_triangle(
    capsule: &mut Capsule,
    triangle: &Triangle,
) -> Option<TriangleIntersection> {
    // Use capsule-plane intersection to find where the capsule hits the triangle's plane
    if let Some(intersection) = intersect_capsule_plane(
        capsule.segment.start,
        capsule.segment.end,
        capsule.radius,
        &triangle.plane,
    ) {
        let t = intersection.t;
        let contact_point = intersection.contact_point;

        // Check if the contact point is inside the triangle
        let barycentric = point_projects_onto_triangle(contact_point, triangle)?;

        // Capsule intersects triangle
        Some(TriangleIntersection { t, barycentric })
    } else {
        None
    }
}

pub fn test_sphere_sphere(a: Sphere, b: Sphere) -> bool {
    let ab = b.center - a.center;

    let r1_r2 = a.radius + b.radius;

    ab.mag_squared() <= r1_r2 * r1_r2
}

pub fn test_moving_spheres(a: Sphere, b: Sphere, velocity: Vec3, velocity_magnitude: f32) -> bool {
    intersect_moving_spheres(a, b, velocity, velocity_magnitude).is_some()
}

/// Tests for collision between two spheres where sphere A is moving with the given velocity.
pub fn intersect_moving_spheres(
    a: Sphere,
    mut b: Sphere,
    velocity: Vec3,
    velocity_magnitude: f32,
) -> Option<ContinuousCollision> {
    // Expands the radius of sphere B by that of sphere A.
    // This allows us to treat sphere A as a point (ray casting optimization).
    b.radius += a.radius;

    let ray = Ray::new(a.center, velocity.as_normal());

    match intersect_ray_sphere(&ray, &b) {
        Some(intersection) => {
            // Only return collision if it occurs within the movement distance
            if intersection.distance <= velocity_magnitude {
                Some(ContinuousCollision {
                    distance: intersection.distance,
                    contact_point: intersection.intersection_point,
                })
            } else {
                None
            }
        }
        None => None,
    }
}

/// Tests for intersection between a moving capsule and a plane.
///
/// A capsule is a sphere swept along a line segment from `start` to `end`.
pub fn intersect_capsule_plane(
    start: Vec3,
    end: Vec3,
    radius: f32,
    plane: &Plane,
) -> Option<CapsulePlaneIntersection> {
    let n = plane.normal;

    let signed_distance = n.dot(start) - plane.d;

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
        // The capsule is already overlapping the plane at start position;
        // set t to zero and contact point to the deepest penetration point.

        Some(CapsulePlaneIntersection {
            t,
            contact_point: start - plane.normal * radius,
        })
    } else {
        // Checks if a collision occurs between start and end positions.

        let v = end - start;

        let n_dot_v = n.dot(v);

        if n_dot_v * signed_distance >= 0.0 {
            // Capsule is moving parallel or away from the plane, so no collision.
            None
        } else {
            // Capsule is moving towards the plane.

            // Chooses a plane offset based on which side of the plane the
            // capsule is initially positioned at.
            let r = if signed_distance > 0.0 {
                radius
            } else {
                -radius
            };

            t = (r - signed_distance) / n_dot_v;

            // Computes the point on the line segment where collision occurs.
            let segment_point = start + v * t;

            // Offsets the segment intersection point towards the plane by radius units
            // to get the actual contact point on the capsule's surface.
            let contact_point = segment_point - n * r;

            Some(CapsulePlaneIntersection { t, contact_point })
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

        let w = 1.0 - u - v;

        ray.collision = Some(StaticBVHCollision {
            t,
            colliding_bvh_index: bvh_instance_index,
            colliding_primitive: triangle_index,
            barycentric: Vec3 { x: u, y: v, z: w },
        });
    }
}

/// Tests for intersection between a ray and a sphere.
pub fn intersect_ray_sphere(ray: &Ray, sphere: &Sphere) -> Option<RaySphereIntersection> {
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
    let mut distance = -b - discriminant.sqrt();

    // A negative t-value indicates that the intersecting ray starts inside of the sphere.
    // Clamp to 0 to return the exit point.
    distance = distance.max(0.0);

    let intersection_point = ray.origin + ray.direction * distance;

    Some(RaySphereIntersection {
        distance,
        intersection_point,
    })
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

pub fn intersect_ray_aabb(ray: &mut Ray, aabb: &AABB) {
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

        ray.collision = transformed_ray.collision;
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

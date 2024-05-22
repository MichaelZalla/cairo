use crate::{
    animation::lerp, scene::camera::frustum::NdcPlane, vertex::default_vertex_out::DefaultVertexOut,
};

use super::Triangle;

fn get_signed_distance_ratio(
    src: &DefaultVertexOut,
    dest: &DefaultVertexOut,
    ndc_plane: NdcPlane,
) -> f32 {
    let (d1, d2) = match ndc_plane {
        NdcPlane::Near => (
            src.position.z + src.position.w,
            dest.position.z + dest.position.w,
        ),
        NdcPlane::Far => (
            src.position.z - src.position.w,
            dest.position.z - dest.position.w,
        ),
        NdcPlane::Left => (
            src.position.x + src.position.w,
            dest.position.x + dest.position.w,
        ),
        NdcPlane::Right => (
            src.position.x - src.position.w,
            dest.position.x - dest.position.w,
        ),
        NdcPlane::Top => (
            src.position.y - src.position.w,
            dest.position.y - dest.position.w,
        ),
        NdcPlane::Bottom => (
            src.position.y + src.position.w,
            dest.position.y + dest.position.w,
        ),
    };

    d1 / (d1 - d2)
}

pub(in crate::pipeline) fn clip_by_all_planes(
    triangle: &Triangle<DefaultVertexOut>,
) -> Vec<Triangle<DefaultVertexOut>> {
    let mut clipped_triangles = vec![*triangle];

    clipped_triangles = clip_triangles_by_plane(NdcPlane::Near, clipped_triangles);
    clipped_triangles = clip_triangles_by_plane(NdcPlane::Far, clipped_triangles);
    clipped_triangles = clip_triangles_by_plane(NdcPlane::Left, clipped_triangles);
    clipped_triangles = clip_triangles_by_plane(NdcPlane::Right, clipped_triangles);
    clipped_triangles = clip_triangles_by_plane(NdcPlane::Top, clipped_triangles);
    clipped_triangles = clip_triangles_by_plane(NdcPlane::Bottom, clipped_triangles);

    clipped_triangles
}

pub(in crate::pipeline) fn clip_triangles_by_plane(
    ndc_plane: NdcPlane,
    triangles: Vec<Triangle<DefaultVertexOut>>,
) -> Vec<Triangle<DefaultVertexOut>> {
    let mut all_clipped = vec![];

    for triangle in triangles {
        for clipped in clip_triangle_by_plane(ndc_plane, triangle) {
            all_clipped.push(clipped);
        }
    }

    all_clipped
}

pub(in crate::pipeline) fn clip_triangle_by_plane(
    ndc_plane: NdcPlane,
    triangle: Triangle<DefaultVertexOut>,
) -> Vec<Triangle<DefaultVertexOut>> {
    // Clip triangles against the near plane (z=0).

    let mut vertices_inside_plane = vec![];
    let mut indices_inside_plane = vec![];

    let mut vertices_outside_plane = vec![];
    let mut indices_outside_plane = vec![];

    for index in 0..3 {
        let v = if index == 0 {
            &triangle.v0
        } else if index == 1 {
            &triangle.v1
        } else {
            &triangle.v2
        };

        let is_inside_plane = match ndc_plane {
            NdcPlane::Near => v.position.z > -v.position.w,
            NdcPlane::Far => v.position.z < v.position.w,
            NdcPlane::Left => v.position.x > -v.position.w,
            NdcPlane::Right => v.position.x < v.position.w,
            NdcPlane::Bottom => v.position.y > -v.position.w,
            NdcPlane::Top => v.position.y < v.position.w,
        };

        if is_inside_plane {
            indices_inside_plane.push(index);
            vertices_inside_plane.push(v);
        } else {
            indices_outside_plane.push(index);
            vertices_outside_plane.push(v);
        }
    }

    if vertices_inside_plane.len() == 3 {
        vec![triangle]
    } else if vertices_outside_plane.len() == 2 {
        // Two points lie outside of the plane; clip 2.

        let a = vertices_inside_plane[0];
        let b = vertices_outside_plane[0];
        let c = vertices_outside_plane[1];

        let a_index = indices_inside_plane[0];
        let b_index = indices_outside_plane[0];

        let b_alpha = get_signed_distance_ratio(b, a, ndc_plane);
        let b_prime = lerp(*b, *a, b_alpha);

        let c_alpha = get_signed_distance_ratio(c, a, ndc_plane);
        let c_prime = lerp(*c, *a, c_alpha);

        if (a_index + 1) % 3 == b_index {
            vec![Triangle {
                v0: *a,
                v1: b_prime,
                v2: c_prime,
            }]
        } else {
            vec![Triangle {
                v0: *a,
                v1: c_prime,
                v2: b_prime,
            }]
        }
    } else if vertices_outside_plane.len() == 1 {
        // One point lies outside of the plane; clip 1.

        let a = vertices_inside_plane[0];
        let b = vertices_outside_plane[0];
        let c = vertices_inside_plane[1];

        let a_index = indices_inside_plane[0];
        let b_index = indices_outside_plane[0];

        let ab_alpha = get_signed_distance_ratio(b, a, ndc_plane);
        let cb_alpha = get_signed_distance_ratio(b, c, ndc_plane);

        let a_prime = lerp(*b, *a, ab_alpha);
        let c_prime = lerp(*b, *c, cb_alpha);

        if (a_index + 1) % 3 == b_index {
            vec![
                Triangle {
                    v0: *a,
                    v1: a_prime,
                    v2: c_prime,
                },
                Triangle {
                    v0: *a,
                    v1: c_prime,
                    v2: *c,
                },
            ]
        } else {
            vec![
                Triangle {
                    v0: *a,
                    v1: *c,
                    v2: c_prime,
                },
                Triangle {
                    v0: *a,
                    v1: c_prime,
                    v2: a_prime,
                },
            ]
        }
    } else {
        // Triangle is entirely outside of the plane.

        vec![]
    }
}

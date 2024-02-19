use crate::{animation::lerp, vertex::default_vertex_out::DefaultVertexOut};

use super::Triangle;

pub(in crate::pipeline) fn clip(
    triangle: &Triangle<DefaultVertexOut>,
) -> Vec<Triangle<DefaultVertexOut>> {
    // Clip triangles that intersect the front of our view frustum

    if triangle.v0.position.z < 0.0 {
        if triangle.v1.position.z < 0.0 {
            // Clip 2 (0 and 1)

            clip2(triangle.v0, triangle.v1, triangle.v2)
        } else if triangle.v2.position.z < 0.0 {
            // Clip 2 (0 and 2)
            clip1(triangle.v0, triangle.v2, triangle.v1)
        } else {
            // Clip 1 (0)
            clip1(triangle.v0, triangle.v1, triangle.v2)
        }
    } else if triangle.v1.position.z < 0.0 {
        if triangle.v2.position.z < 0.0 {
            // Clip 2
            clip2(triangle.v1, triangle.v2, triangle.v0)
        } else {
            // Clip 1
            clip1(triangle.v1, triangle.v0, triangle.v2)
        }
    } else if triangle.v2.position.z < 0.0 {
        // Clip 1
        clip1(triangle.v2, triangle.v0, triangle.v1)
    } else {
        vec![*triangle]
    }
}

fn clip1(
    v0: DefaultVertexOut,
    v1: DefaultVertexOut,
    v2: DefaultVertexOut,
) -> Vec<Triangle<DefaultVertexOut>> {
    let a_alpha = -(v0.position.z) / (v1.position.z - v0.position.z);
    let b_alpha = -(v0.position.z) / (v2.position.z - v0.position.z);

    let a_prime = lerp(v0, v1, a_alpha);
    let b_prime = lerp(v0, v2, b_alpha);

    let triangle1 = Triangle {
        v0: a_prime,
        v1,
        v2,
    };

    let triangle2 = Triangle {
        v0: b_prime,
        v1: a_prime,
        v2,
    };

    vec![triangle1, triangle2]
}

fn clip2(
    v0: DefaultVertexOut,
    v1: DefaultVertexOut,
    v2: DefaultVertexOut,
) -> Vec<Triangle<DefaultVertexOut>> {
    let a_alpha = -(v0.position.z) / (v2.position.z - v0.position.z);
    let b_alpha = -(v1.position.z) / (v2.position.z - v1.position.z);

    let a_prime = lerp(v0, v2, a_alpha);
    let b_prime = lerp(v1, v2, b_alpha);

    let triangle = Triangle {
        v0: a_prime,
        v1: b_prime,
        v2,
    };

    vec![triangle]
}

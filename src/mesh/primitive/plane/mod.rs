use crate::vec::{vec2::Vec2, vec3::Vec3, vec4};

use super::{Face, Mesh};

pub fn generate(width: f32, depth: f32, width_divisions: u32, depth_divisions: u32) -> Mesh {
    assert!(width_divisions >= 1 && depth_divisions >= 1);

    // Generate vertices and UVs

    let mut vertices: Vec<Vec3> = vec![];

    let mut uvs: Vec<Vec2> = vec![];

    for z in 0..depth_divisions + 1 {
        for x in 0..width_divisions + 1 {
            let x_alpha = x as f32 * (1.0 / width_divisions as f32);
            let z_alpha = z as f32 * (1.0 / depth_divisions as f32);

            vertices.push(Vec3 {
                x: (-width / 2.0) + width * x_alpha,
                y: 0.0,
                z: (-depth / 2.0) + depth * z_alpha,
            });

            uvs.push(Vec2 {
                x: x_alpha,
                y: (1.0 - z_alpha),
                z: 0.0,
            });
        }
    }

    assert!(vertices.len() as u32 == (width_divisions + 1) * (depth_divisions + 1));
    assert!(uvs.len() == vertices.len());

    // Generate normals

    let up = Vec3 {
        x: vec4::UP.x,
        y: vec4::UP.y,
        z: vec4::UP.z,
    };

    let normals = vec![up];

    // Generate faces

    let mut faces: Vec<Face> = vec![];

    let pitch = width_divisions + 1;

    for z in 0..depth_divisions {
        for x in 0..width_divisions {
            let face_1 = Face {
                // (far_left, far_right, near_left) (clockwise)
                vertices: (
                    (z * pitch + x) as usize,
                    (z * pitch + x + 1) as usize,
                    ((z + 1) * pitch + x) as usize,
                ),
                // (top_left, top_right, bottom_left) (clockwise)
                uvs: Some((
                    (z * pitch + x) as usize,
                    (z * pitch + x + 1) as usize,
                    ((z + 1) * pitch + x) as usize,
                )),
                // (up, up, up)
                normals: Some((0, 0, 0)),
            };

            let face_2 = Face {
                // (near_left, far_right, near_right) (clockwise)
                vertices: (
                    ((z + 1) * pitch + x) as usize,
                    (z * pitch + x + 1) as usize,
                    ((z + 1) * pitch + x + 1) as usize,
                ),
                // (bottom_left, top_right, bottom_right) (clockwise)
                uvs: Some((
                    ((z + 1) * pitch + x) as usize,
                    (z * pitch + x + 1) as usize,
                    ((z + 1) * pitch + x + 1) as usize,
                )),
                // (up, up, up)
                normals: Some((0, 0, 0)),
            };

            faces.push(face_1);
            faces.push(face_2);
        }
    }

    return Mesh::new(vertices, uvs, normals, faces);
}
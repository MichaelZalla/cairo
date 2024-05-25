use std::rc::Rc;

use crate::{
    mesh::{geometry::Geometry, Mesh, PartialFace},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

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
                y: z_alpha,
                z: 0.0,
            });
        }
    }

    assert!(vertices.len() as u32 == (width_divisions + 1) * (depth_divisions + 1));
    assert!(uvs.len() == vertices.len());

    // Generate normals

    let up = vec3::UP;

    let normals = vec![up];

    // Generate faces

    let mut partial_faces: Vec<PartialFace> = vec![];

    let pitch = width_divisions + 1;

    for z in 0..depth_divisions {
        for x in 0..width_divisions {
            let face_1 = PartialFace {
                // (near_left, far_right, far_left) (counter-clockwise)
                vertices: [
                    ((z + 1) * pitch + x) as usize,
                    (z * pitch + x + 1) as usize,
                    (z * pitch + x) as usize,
                ],
                // (bottom_left, top_right, top_left) (counter-clockwise)
                uvs: Some([
                    ((z + 1) * pitch + x) as usize,
                    (z * pitch + x + 1) as usize,
                    (z * pitch + x) as usize,
                ]),
                // (up, up, up)
                normals: Some([0, 0, 0]),
            };

            let face_2 = PartialFace {
                // (near_left, far_right, near_right) (counter-clockwise)
                vertices: [
                    ((z + 1) * pitch + x + 1) as usize,
                    (z * pitch + x + 1) as usize,
                    ((z + 1) * pitch + x) as usize,
                ],
                // (bottom_left, top_right, bottom_right) (counter-clockwise)
                uvs: Some([
                    ((z + 1) * pitch + x + 1) as usize,
                    (z * pitch + x + 1) as usize,
                    ((z + 1) * pitch + x) as usize,
                ]),
                // (up, up, up)
                normals: Some([0, 0, 0]),
            };

            partial_faces.push(face_1);
            partial_faces.push(face_2);
        }
    }

    let geometry = Geometry {
        vertices,
        uvs,
        normals,
    };

    let mut mesh = Mesh::new(Rc::new(geometry), partial_faces, None);

    mesh.object_name = Some("plane".to_string());

    mesh
}

use std::{f32::consts::PI, rc::Rc};

use crate::{
    mesh::{geometry::Geometry, Mesh, PartialFace},
    vec::{
        vec2::{self, Vec2},
        vec3::{self, Vec3},
    },
};

use crate::texture;

pub fn generate(radius: f32, height: f32, divisions: u32) -> Mesh {
    assert!(divisions >= 3);

    // Generate vertices and UVs

    let top_center_vertex = vec3::UP * height / 2.0;

    let bottom_center_vertex = vec3::UP * -height / 2.0;

    let center_uv = vec2::Vec2::interpolate(texture::uv::TOP_LEFT, texture::uv::BOTTOM_RIGHT, 0.5);

    let mut ring_vertices: Vec<Vec3> = vec![];
    let mut ring_uvs: Vec<Vec2> = vec![];

    for i in 0..divisions + 1 {
        // Generate vertices and UVs around the base
        let alpha: f32 = i as f32 * (1.0 / divisions as f32);
        let radians = 2.0 * PI * alpha;

        ring_vertices.push(Vec3 {
            x: (radius / 2.0) * radians.cos(),
            y: bottom_center_vertex.y,
            z: (radius / 2.0) * radians.sin(),
        });

        ring_uvs.push(Vec2 {
            x: radians.cos() / 2.0 + 0.5,
            y: radians.sin() / 2.0 + 0.5,
            z: 0.0,
        });
    }

    assert!(ring_vertices.len() as u32 == divisions + 1);
    assert!(ring_uvs.len() == ring_vertices.len());

    let mut vertices: Vec<Vec3> = vec![];

    vertices.append(&mut ring_vertices);
    vertices.append(&mut vec![bottom_center_vertex, top_center_vertex]);

    let bottom_center_index = (divisions + 1) as usize;
    let top_center_index = (divisions + 2) as usize;

    assert!(top_center_index == vertices.len() - 1);

    let mut uvs: Vec<Vec2> = vec![];

    uvs.append(&mut ring_uvs);
    uvs.append(&mut vec![center_uv]);

    let center_uv_index = uvs.len() - 1_usize;

    // Generate normals

    let up = vec3::UP;

    let down = up * -1.0;

    let mut normals = vec![down];

    // Generate faces

    let mut partial_faces: Vec<PartialFace> = vec![];

    for i in 0..divisions {
        // Generate a ring of faces around the base

        partial_faces.push(PartialFace {
            // (ring_i, ring_i + 1, bottom_center) (clockwise)
            vertices: [i as usize, i as usize + 1, bottom_center_index],
            // (down, down, down)
            normals: Some([0, 0, 0]),
            // (ring_i, ring_i + 1, center) (clockwise)
            uvs: Some([i as usize, i as usize + 1, center_uv_index]),
        });

        // (ring_i + 1, ring_i, top_center) (counter-clockwise)
        let vertex_indices = [i as usize + 1, i as usize, top_center_index];

        // @TODO Smooth normals for cone sides
        normals.push(
            (vertices[vertex_indices[1]] - vertices[vertex_indices[0]])
                .cross(vertices[vertex_indices[2]] - vertices[vertex_indices[0]])
                .as_normal(),
        );

        let normal_index = normals.len() - 1;

        partial_faces.push(PartialFace {
            vertices: vertex_indices,
            // (normal to the face)
            normals: Some([normal_index, normal_index, normal_index]),
            // (ring_i + 1, ring_i, center) (counter-clockwise)
            uvs: Some([i as usize + 1, i as usize, center_uv_index]),
        });
    }

    let geometry = Geometry {
        vertices,
        uvs,
        normals,
    };

    let mut mesh = Mesh::new(Rc::new(geometry), partial_faces, None);

    mesh.object_name = Some("cone".to_string());

    mesh
}

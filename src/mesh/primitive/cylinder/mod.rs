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

    let bottom_center_vertex = top_center_vertex * -1.0;

    let center_uv = vec2::Vec2::interpolate(texture::uv::TOP_LEFT, texture::uv::BOTTOM_RIGHT, 0.5);

    let mut top_ring_vertices: Vec<Vec3> = vec![];
    let mut bottom_ring_vertices: Vec<Vec3> = vec![];

    let mut top_ring_uvs: Vec<Vec2> = vec![];
    let mut bottom_ring_uvs: Vec<Vec2> = vec![];

    for i in 0..divisions + 1 {
        // Generate vertices and UVs around the base
        let alpha: f32 = i as f32 * (1.0 / divisions as f32);
        let radians = 2.0 * PI * alpha;

        top_ring_vertices.push(Vec3 {
            x: (radius / 2.0) * radians.cos(),
            y: top_center_vertex.y,
            z: (radius / 2.0) * radians.sin(),
        });

        top_ring_uvs.push(Vec2 {
            x: -radians.cos() / 2.0 + 0.5,
            y: -radians.sin() / 2.0 + 0.5,
            z: 0.0,
        });

        bottom_ring_vertices.push(Vec3 {
            x: (radius / 2.0) * radians.cos(),
            y: bottom_center_vertex.y,
            z: (radius / 2.0) * radians.sin(),
        });

        bottom_ring_uvs.push(Vec2 {
            x: radians.cos() / 2.0 + 0.5,
            y: radians.sin() / 2.0 + 0.5,
            z: 0.0,
        });
    }

    assert!(top_ring_vertices.len() as u32 == divisions + 1);
    assert!(bottom_ring_vertices.len() as u32 == divisions + 1);

    assert!(top_ring_uvs.len() == top_ring_vertices.len());
    assert!(bottom_ring_uvs.len() == bottom_ring_vertices.len());

    let mut vertices: Vec<Vec3> = vec![];

    vertices.append(&mut top_ring_vertices);
    vertices.append(&mut vec![top_center_vertex]);
    vertices.append(&mut bottom_ring_vertices);
    vertices.append(&mut vec![bottom_center_vertex]);

    let top_center_index = (divisions + 1) as usize;
    let bottom_center_index = vertices.len() - 1_usize;

    let mut uvs: Vec<Vec2> = vec![];

    uvs.append(&mut top_ring_uvs);
    uvs.append(&mut bottom_ring_uvs);
    uvs.append(&mut vec![center_uv]);

    let center_uv_index = uvs.len() - 1_usize;

    let uv_strips_start_index = uvs.len();

    // Generate normals

    let up = vec3::UP;

    let down = up * -1.0;

    let mut normals = vec![up, down];

    // Generate faces

    let mut partial_faces: Vec<PartialFace> = vec![];

    for i in 0..divisions as usize {
        // Vertex indices

        let top_center = top_center_index;
        let top_ring_left = i;
        let top_ring_right = i + 1;

        let bottom_center = bottom_center_index;
        let bottom_ring_left = divisions as usize + 2 + i;
        let bottom_ring_right = divisions as usize + 2 + i + 1;

        // Generate a top face

        partial_faces.push(PartialFace {
            // (top_ring_right, top_ring_left, top_center) (counter-clockwise)
            vertices: [top_ring_right, top_ring_left, top_center],
            // (up, up, up)
            normals: Some([0, 0, 0]),
            // (ring_i + 1, ring_i, center) (counter-clockwise)
            uvs: Some([i + 1, i, center_uv_index]),
        });

        // Generate a bottom face
        partial_faces.push(PartialFace {
            // (bottom_ring_left, bottom_ring_right, bottom_center) (counter-clockwise)
            vertices: [bottom_ring_left, bottom_ring_right, bottom_center],
            // (down, down, down)
            normals: Some([1, 1, 1]),
            // (ring_i, ring_i + 1, center) (counter-clockwise)
            uvs: Some([i, i + 1, center_uv_index]),
        });

        // Generate 2 side faces (for each quad)

        // How far along the U-axis of our UV texture space
        let uv_strip_width = 1.0 / divisions as f32;
        let alpha: f32 = i as f32 * uv_strip_width;

        let uv_strip_top_left = Vec2 {
            x: PI * alpha,
            y: 1.0,
            z: 0.0,
        };

        let uv_strip_top_right = Vec2 {
            x: PI * alpha + uv_strip_width,
            y: 1.0,
            z: 0.0,
        };

        let uv_strip_bottom_left = Vec2 {
            x: PI * alpha,
            y: 0.0,
            z: 0.0,
        };

        let uv_strip_bottom_right = Vec2 {
            x: PI * alpha + uv_strip_width,
            y: 0.0,
            z: 0.0,
        };

        uvs.append(&mut vec![
            uv_strip_top_left,
            uv_strip_top_right,
            uv_strip_bottom_left,
            uv_strip_bottom_right,
        ]);

        let uv_strip_top_left_index = uv_strips_start_index + i * 4;
        let uv_strip_top_right_index = uv_strips_start_index + i * 4 + 1;
        let uv_strip_bottom_left_index = uv_strips_start_index + i * 4 + 2;
        let uv_strip_bottom_right_index = uv_strips_start_index + i * 4 + 3;

        // (top_ring_right, bottom_ring_left, top_ring_left) (counter-clockwise)
        let vertex_indices_1 = [top_ring_right, bottom_ring_left, top_ring_left];

        // @TODO Smooth normals for cylinder sides
        normals.push(
            (vertices[vertex_indices_1[1]] - vertices[vertex_indices_1[0]])
                .cross(vertices[vertex_indices_1[2]] - vertices[vertex_indices_1[0]])
                .as_normal(),
        );

        let mut normal_index = normals.len() - 1;

        partial_faces.push(PartialFace {
            vertices: vertex_indices_1,
            // (normal to the face)
            normals: Some([normal_index, normal_index, normal_index]),
            // (uv_strip_top_right, uv_strip_bottom_left, uv_strip_top_left) (counter-clockwise)
            uvs: Some([
                uv_strip_top_right_index,
                uv_strip_bottom_left_index,
                uv_strip_top_left_index,
            ]),
        });

        // (top_ring_right, bottom_ring_right, bottom_ring_left) (counter-clockwise)
        let vertex_indices_2 = [top_ring_right, bottom_ring_right, bottom_ring_left];

        // @TODO Smooth normals for cylinder sides
        normals.push(
            (vertices[vertex_indices_2[1]] - vertices[vertex_indices_2[0]])
                .cross(vertices[vertex_indices_2[2]] - vertices[vertex_indices_2[0]])
                .as_normal(),
        );

        normal_index = normals.len() - 1;

        partial_faces.push(PartialFace {
            vertices: vertex_indices_2,
            // (normal to the face)
            normals: Some([normal_index, normal_index, normal_index]),
            // (uv_strip_top_right, uv_strip_bottom_right, uv_strip_bottom_left) (counter-clockwise)
            uvs: Some([
                uv_strip_top_right_index,
                uv_strip_bottom_right_index,
                uv_strip_bottom_left_index,
            ]),
        });
    }

    let geometry = Geometry {
        vertices: vertices.into_boxed_slice(),
        uvs: uvs.into_boxed_slice(),
        normals: normals.into_boxed_slice(),
    };

    let mut mesh = Mesh::new(Rc::new(geometry), partial_faces, None);

    mesh.object_name = Some("cylinder".to_string());

    mesh
}

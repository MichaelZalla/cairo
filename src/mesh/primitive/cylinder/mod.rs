use std::f32::consts::PI;

use crate::vec::{
    vec2::{self, Vec2},
    vec3::Vec3,
    vec4,
};

use super::{Face, Mesh};

use crate::image;

pub fn generate(radius: f32, height: f32, divisions: u32) -> Mesh {
    assert!(divisions >= 3);

    // Generate vertices and UVs

    let top_center_vertex = Vec3 {
        x: vec4::UP.x,
        y: vec4::UP.y,
        z: vec4::UP.z,
    } * height
        / 2.0;

    let bottom_center_vertex = top_center_vertex * -1.0;

    let center_uv = vec2::Vec2::interpolate(image::uv::TOP_LEFT, image::uv::BOTTOM_RIGHT, 0.5);

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
    let bottom_center_index = vertices.len() - 1 as usize;

    let mut uvs: Vec<Vec2> = vec![];

    uvs.append(&mut top_ring_uvs);
    uvs.append(&mut bottom_ring_uvs);
    uvs.append(&mut vec![center_uv]);

    let center_uv_index = uvs.len() - 1 as usize;

    let uv_strips_start_index = uvs.len();

    // Generate normals

    let up = Vec3 {
        x: vec4::UP.x,
        y: vec4::UP.y,
        z: vec4::UP.z,
    };

    let down = up * -1.0;

    let mut normals = vec![up, down];

    // Generate faces

    let mut faces: Vec<Face> = vec![];

    for i in 0..divisions as usize {
        // Vertex indices

        let top_center = top_center_index;
        let top_ring_left = i as usize;
        let top_ring_right = i as usize + 1;

        let bottom_center = bottom_center_index;
        let bottom_ring_left = divisions as usize + 2 + i as usize;
        let bottom_ring_right = divisions as usize + 2 + i as usize + 1;

        // Generate a top face

        faces.push(Face {
            // (top_center, top_ring_left, top_ring_right) (clockwise)
            vertices: (top_center, top_ring_left, top_ring_right),
            // (up, up, up)
            normals: Some((0, 0, 0)),
            // (center, ring_i, ring_i + 1) (clockwise)
            uvs: Some((center_uv_index, i as usize, i as usize + 1)),
        });

        // Generate a bottom face
        faces.push(Face {
            // (bottom_center, bottom_ring_right, bottom_ring_left) (clockwise)
            vertices: (bottom_center, bottom_ring_right, bottom_ring_left),
            // (down, down, down)
            normals: Some((1, 1, 1)),
            // (center, ring_i + 1, ring_i) (clockwise)
            uvs: Some((center_uv_index, i as usize + 1, i as usize)),
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

        // (top_ring_left, bottom_ring_left, top_ring_right) (clockwise)
        let vertex_indices_1 = (top_ring_left, bottom_ring_left, top_ring_right);

        // @TODO Smooth normals for cylinder sides
        normals.push(
            (vertices[vertex_indices_1.1] - vertices[vertex_indices_1.0])
                .cross(vertices[vertex_indices_1.2] - vertices[vertex_indices_1.0])
                .as_normal(),
        );

        let mut normal_index = normals.len() - 1;

        faces.push(Face {
            vertices: vertex_indices_1,
            // (normal to the face)
            normals: Some((normal_index, normal_index, normal_index)),
            // (uv_strip_top_left, uv_strip_bottom_left, uv_strip_top_right) (clockwise)
            uvs: Some((
                uv_strip_top_left_index,
                uv_strip_bottom_left_index,
                uv_strip_top_right_index,
            )),
        });

        // (bottom_ring_left, bottom_ring_right, top_ring_right) (clockwise)
        let vertex_indices_2 = (bottom_ring_left, bottom_ring_right, top_ring_right);

        // @TODO Smooth normals for cylinder sides
        normals.push(
            (vertices[vertex_indices_2.1] - vertices[vertex_indices_2.0])
                .cross(vertices[vertex_indices_2.2] - vertices[vertex_indices_2.0])
                .as_normal(),
        );

        normal_index = normals.len() - 1;

        faces.push(Face {
            vertices: vertex_indices_2,
            // (normal to the face)
            normals: Some((normal_index, normal_index, normal_index)),
            // (uv_strip_bottom_left, uv_strip_bottom_right, uv_strip_top_right) (clockwise)
            uvs: Some((
                uv_strip_bottom_left_index,
                uv_strip_bottom_right_index,
                uv_strip_top_right_index,
            )),
        });
    }

    return Mesh::new(vertices, uvs, normals, faces);
}

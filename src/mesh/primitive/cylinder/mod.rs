use std::{f32::consts::TAU, mem, rc::Rc};

use crate::{
    mesh::{geometry::Geometry, Mesh, PartialFace},
    texture::uv,
    transform::quaternion::Quaternion,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4,
    },
};

pub fn generate(radius: f32, height: f32, divisions: u32) -> Mesh {
    assert!(divisions >= 3);

    let mut normals = vec![vec3::UP, -vec3::UP];

    let mut vertices = vec![];

    let mut uvs = vec![];

    let mut partial_faces = vec![];

    let alpha_step = 1.0 / divisions as f32;

    let height_over_2: f32 = height / 2.0;

    let center_top = Vec3 {
        x: 0.0,
        y: height_over_2,
        z: 0.0,
    };

    let center_bottom = Vec3 {
        x: 0.0,
        y: -height_over_2,
        z: 0.0,
    };

    for i in 0..divisions as usize {
        let alpha = alpha_step * i as f32;

        let rotation = Quaternion::new(vec3::UP, TAU * -alpha);

        // Normal

        let side_edge_normal = (vec4::RIGHT * (*rotation.mat())).to_vec3();

        normals.push(side_edge_normal);

        // Vertices

        let side_edge_vertex = side_edge_normal * radius;

        let side_edge_vertex_top = side_edge_vertex + center_top;
        let side_edge_vertex_bottom = side_edge_vertex + center_bottom;

        vertices.push(side_edge_vertex_top);
        vertices.push(side_edge_vertex_bottom);

        // UVs

        let side_edge_uv_top = uv::TOP_LEFT + (uv::TOP_RIGHT - uv::TOP_LEFT) * alpha;

        let mut side_edge_uv_bottom = side_edge_uv_top;
        side_edge_uv_bottom.y = uv::BOTTOM_LEFT.y;

        uvs.push(side_edge_uv_top);
        uvs.push(side_edge_uv_bottom);

        // Partial faces

        if i > 0 {
            side_triangle_strip_between_i_j(i - 1, i, &mut partial_faces);

            if i == divisions as usize - 1 {
                side_triangle_strip_between_i_j(i, 0, &mut partial_faces);
            }
        }
    }

    // Cap UVs.

    let uvs_center_index = uvs.len();

    uvs.push(uv::CENTER);

    let uvs_ring_start_index = uvs.len();

    for i in 0..divisions as usize {
        let alpha = alpha_step * i as f32;

        let rotation = Quaternion::new(vec3::FORWARD, TAU * alpha);

        let uv_normal = vec4::RIGHT * (*rotation.mat());

        let uv = Vec2 {
            x: 0.5 + uv_normal.x,
            y: 0.5 + uv_normal.y,
            z: 0.0,
        };

        uvs.push(uv);
    }

    // Top cap.

    let mut center_vertex_index = vertices.len();

    vertices.push(center_top);

    do_triangle_fan_cap(
        divisions as usize,
        center_vertex_index,
        uvs_center_index,
        uvs_ring_start_index,
        true,
        &mut partial_faces,
    );

    // Bottom cap.

    center_vertex_index = vertices.len();

    vertices.push(center_bottom);

    do_triangle_fan_cap(
        divisions as usize,
        center_vertex_index,
        uvs_center_index,
        uvs_ring_start_index,
        false,
        &mut partial_faces,
    );

    // Packaging.

    let geometry = Geometry {
        vertices: vertices.into_boxed_slice(),
        uvs: uvs.into_boxed_slice(),
        normals: normals.into_boxed_slice(),
    };

    let mut mesh = Mesh::new(Rc::new(geometry), partial_faces, None);

    mesh.object_name = Some("cylinder".to_string());

    mesh
}

fn side_triangle_strip_between_i_j(i: usize, j: usize, partial_faces: &mut Vec<PartialFace>) {
    // Offset to account for "up" and "down" normals that begin our normals.
    let (n_i, n_j) = (i + 2, j + 2);

    // Index of the first vertex associated with each side (edge).
    let (v_i, v_j) = (i * 2, j * 2);

    let (v_1, v_2, v_3, v_4) = (v_i, v_i + 1, v_j, v_j + 1);

    let side_strip_face_1 = PartialFace {
        normals: Some([n_i, n_i, n_j]),
        vertices: [v_1, v_2, v_3],
        uvs: Some([v_1, v_2, v_3]),
    };

    let side_strip_face_2 = PartialFace {
        normals: Some([n_j, n_j, n_i]),
        vertices: [v_4, v_3, v_2],
        uvs: Some([v_4, v_3, v_2]),
    };

    partial_faces.push(side_strip_face_1);
    partial_faces.push(side_strip_face_2);
}

fn do_triangle_fan_cap(
    divisions: usize,
    center_vertex_index: usize,
    uvs_center_index: usize,
    uvs_ring_start_index: usize,
    is_top: bool,
    partial_faces: &mut Vec<PartialFace>,
) {
    for i in 0..divisions {
        // Index of the top or bottom vertex associated with each side (edge).
        let (mut v_i, mut v_j) = (i * 2, (i + 1) * 2);

        if !is_top {
            v_i += 1;
            v_j += 1;
        }

        let (mut uv_i, mut uv_j) = (uvs_ring_start_index + i, uvs_ring_start_index + i + 1);

        if i == divisions - 1 {
            v_j = if is_top { 0 } else { 1 };
            uv_j = uvs_ring_start_index;
        }

        if !is_top {
            mem::swap(&mut v_i, &mut v_j);
            mem::swap(&mut uv_i, &mut uv_j);
        }

        partial_faces.push(PartialFace {
            normals: Some(if is_top { [0, 0, 0] } else { [1, 1, 1] }),
            vertices: [v_i, v_j, center_vertex_index],
            uvs: Some([uv_i, uv_j, uvs_center_index]),
        });
    }
}

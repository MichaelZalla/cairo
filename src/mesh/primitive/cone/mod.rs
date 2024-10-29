use std::{f32::consts::TAU, mem, rc::Rc};

use crate::{
    mesh::{geometry::Geometry, Mesh, PartialFace},
    texture::uv,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4,
    },
};

pub fn generate(radius: f32, height: f32, divisions: u32) -> Mesh {
    assert!(
        divisions >= 3,
        "Called cone::generate() with fewer than 3 divisions!"
    );

    let mut normals = vec![vec3::UP, -vec3::UP];

    let mut vertices = vec![];

    let mut uvs = vec![];

    let mut partial_faces = vec![];

    let alpha_step = 1.0 / divisions as f32;

    let height_over_2 = height / 2.0;

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

    let uvs_center_index = uvs.len();

    uvs.push(uv::CENTER);

    let uvs_ring_start_index = uvs.len();

    for i in 0..divisions as usize {
        let alpha = alpha_step * i as f32;

        let rotation_y = Quaternion::new(vec3::UP, TAU * -alpha);

        // Vertex

        let ring_vertex = (vec4::RIGHT * (*rotation_y.mat())).to_vec3() * radius + center_bottom;

        // Normal

        let normal = {
            let mut normal = (center_top - ring_vertex).as_normal();

            let tangent = {
                let mut tangent = vec3::UP.cross(normal).as_normal();

                if tangent.x.is_nan() {
                    tangent = vec3::RIGHT.cross(normal).as_normal();
                }

                tangent
            };

            let mut bitangent = normal.cross(tangent);

            mem::swap(&mut normal, &mut bitangent);

            normal
        };

        normals.push(normal);

        vertices.push(ring_vertex);

        // UV

        let uv_normal_rotation = Quaternion::new(vec3::FORWARD, TAU * alpha);

        let uv_normal = vec4::RIGHT * (*uv_normal_rotation.mat());

        let uv = uv_normal.ndc_to_uv();

        uvs.push(uv);
    }

    // Cone top.

    let center_vertex_index = vertices.len();

    vertices.push(center_top);

    do_triangle_fan(
        divisions as usize,
        center_vertex_index,
        uvs_center_index,
        uvs_ring_start_index,
        true,
        &mut partial_faces,
    );

    // Bottom cap.

    let center_vertex_index = vertices.len();

    vertices.push(center_bottom);

    do_triangle_fan(
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

    mesh.object_name = Some("cone".to_string());

    mesh
}

fn do_triangle_fan(
    divisions: usize,
    center_vertex_index: usize,
    uvs_center_index: usize,
    uvs_ring_start_index: usize,
    is_top: bool,
    partial_faces: &mut Vec<PartialFace>,
) {
    for i in 0..divisions {
        let n_i = 2 + i;
        let mut n_j = n_i + 1;

        // Index of the top or bottom vertex associated with each side (edge).

        let (mut v_i, mut v_j) = (i, i + 1);

        let (mut uv_i, mut uv_j) = (uvs_ring_start_index + i, uvs_ring_start_index + i + 1);

        if i == divisions - 1 {
            n_j = 2;
            v_j = 0;
            uv_j = uvs_ring_start_index;
        }

        if !is_top {
            mem::swap(&mut v_i, &mut v_j);
            mem::swap(&mut uv_i, &mut uv_j);
        }

        partial_faces.push(PartialFace {
            normals: Some(if !is_top { [1, 1, 1] } else { [n_i, n_j, 0] }),
            vertices: [v_i, v_j, center_vertex_index],
            uvs: Some([uv_i, uv_j, uvs_center_index]),
        });
    }
}

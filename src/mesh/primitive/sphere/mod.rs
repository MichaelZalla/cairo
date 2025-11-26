use std::{
    f32::consts::{PI, TAU},
    rc::Rc,
};

use crate::{
    mesh::{Mesh, PartialFace, mesh_geometry::MeshGeometry, primitive::plane},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

pub fn generate(radius: f32, latitude_divisions: u32, longitude_divisions: u32) -> Mesh {
    assert!(
        latitude_divisions >= 1,
        "Called sphere::generate() with fewer than 2 latitude divisions!"
    );

    assert!(
        longitude_divisions >= 1,
        "Called sphere::generate() with fewer than 2 longitude divisions!"
    );

    let plane_mesh = plane::generate(1.0, 1.0, longitude_divisions, latitude_divisions);

    let geometry = Rc::into_inner(plane_mesh.geometry).unwrap();

    let vertices_boxed_slice = geometry.vertices.to_owned();

    let mut vertices = vertices_boxed_slice.to_vec();

    let mut partial_faces: Vec<PartialFace> = plane_mesh
        .faces
        .iter()
        .map(|face| PartialFace {
            vertices: face.vertices,
            uvs: Some(face.uvs),
            normals: Some(face.normals),
        })
        .collect();

    // Transform the plane (manifold) into a sphere.

    // Formula:
    //
    //   v_i = vertices[longitude_index * longitude_divisions + latitude_index];
    //

    let rotation_x = Quaternion::new(vec3::RIGHT, PI / 2.0);

    let stride = longitude_divisions + 1;

    let x_step = 1.0 / latitude_divisions as f32;
    let y_step = 1.0 / longitude_divisions as f32;

    let pi_over_2 = PI / 2.0;

    for (i, vertex) in vertices.iter_mut().enumerate() {
        let x = i as u32 % stride;
        let y = i as u32 / stride;

        *vertex *= *rotation_x.mat();

        let x_alpha = x_step * x as f32;
        let y_alpha = y_step * y as f32;

        // PI / 2 -> -PI / 2

        let theta_z = pi_over_2 - PI * y_alpha;

        vertex.x = theta_z.cos();
        vertex.z = 0.0;
        vertex.y = theta_z.sin();

        let rotation_y = Quaternion::new(vec3::UP, TAU * x_alpha);

        *vertex *= *rotation_y.mat();
    }

    // Regenerate normals.

    let normals: Vec<Vec3> = vertices.iter().map(|v| v.as_normal()).collect();

    let uvs_boxed_slice = geometry.uvs.to_owned();

    let uvs = uvs_boxed_slice.to_vec();

    for face in partial_faces.iter_mut() {
        face.vertices.reverse();

        face.uvs = Some(face.vertices);

        for vertex_index in face.vertices.iter_mut() {
            let latitude_index = *vertex_index as u32 % stride;

            if latitude_index == latitude_divisions {
                *vertex_index -= latitude_divisions as usize;
            }
        }

        face.normals = Some(face.vertices);
    }

    // Scale geometry by sphere radius.

    for vertex in vertices.iter_mut() {
        *vertex *= radius;
    }

    // Package the geometry.

    let geometry = MeshGeometry {
        vertices: vertices.into_boxed_slice(),
        uvs: uvs.into_boxed_slice(),
        normals: normals.into_boxed_slice(),
    };

    let mut mesh = Mesh::new(Rc::new(geometry), partial_faces, None);

    mesh.object_name = Some("sphere".to_string());

    mesh
}

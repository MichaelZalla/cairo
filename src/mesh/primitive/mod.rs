use crate::vec::vec3::Vec3;

use super::{Face, Mesh};

pub fn make_box(width: f32, height: f32, depth: f32) -> Mesh {
    let front_top_left = Vec3 {
        x: -width / 2.0,
        y: height / 2.0,
        z: depth / 2.0,
    };

    let front_top_right = Vec3 {
        x: width / 2.0,
        y: height / 2.0,
        z: depth / 2.0,
    };

    let front_bottom_left = Vec3 {
        x: -width / 2.0,
        y: -height / 2.0,
        z: depth / 2.0,
    };

    let front_bottom_right = Vec3 {
        x: width / 2.0,
        y: -height / 2.0,
        z: depth / 2.0,
    };

    let mut back_top_left = front_top_left.clone();

    back_top_left.z -= depth;

    let mut back_top_right = front_top_right.clone();

    back_top_right.z -= depth;

    let mut back_bottom_left = front_bottom_left.clone();

    back_bottom_left.z -= depth;

    let mut back_bottom_right = front_bottom_right.clone();

    back_bottom_right.z -= depth;

    let vertices: Vec<Vec3> = vec![
        front_top_left,     // 0
        front_top_right,    // 1
        front_bottom_left,  // 2
        front_bottom_right, // 3
        back_top_left,      // 4
        back_top_right,     // 5
        back_bottom_left,   // 6
        back_bottom_right,  // 7
    ];

    let mut faces: Vec<Face> = vec![];

    // Front face

    faces.push(Face {
        vertices: (0, 2, 1),
        normals: None,
        uvs: None,
    });

    faces.push(Face {
        vertices: (2, 3, 1),
        normals: None,
        uvs: None,
    });

    // Back face

    faces.push(Face {
        vertices: (4, 5, 6),
        normals: None,
        uvs: None,
    });

    faces.push(Face {
        vertices: (5, 7, 6),
        normals: None,
        uvs: None,
    });

    // Top face

    faces.push(Face {
        vertices: (4, 0, 5),
        normals: None,
        uvs: None,
    });

    faces.push(Face {
        vertices: (0, 1, 5),
        normals: None,
        uvs: None,
    });

    // Bottom face

    faces.push(Face {
        vertices: (6, 7, 2),
        normals: None,
        uvs: None,
    });

    faces.push(Face {
        vertices: (7, 3, 2),
        normals: None,
        uvs: None,
    });

    // Left face

    faces.push(Face {
        vertices: (4, 6, 0),
        normals: None,
        uvs: None,
    });

    faces.push(Face {
        vertices: (6, 2, 0),
        normals: None,
        uvs: None,
    });

    // Right face

    faces.push(Face {
        vertices: (1, 3, 7),
        normals: None,
        uvs: None,
    });

    faces.push(Face {
        vertices: (7, 5, 1),
        normals: None,
        uvs: None,
    });

    return Mesh::new(vertices, vec![], vec![], faces);
}

use sdl2::sys::SDL_STANDARD_GRAVITY;

use cairo::vec::vec3::Vec3;

use crate::collider::LineSegmentCollider;
use crate::force::{Force, Newtons};
use crate::simulation::Simulation;
use crate::springy_mesh::SpringyMesh;
use crate::state_vector::StateVector;

static GRAVITY: Force = |_state: &StateVector, _i: usize, _current_time: f32| -> Newtons {
    Vec3 {
        x: 0.0,
        y: -(SDL_STANDARD_GRAVITY as f32),
        z: 0.0,
    }
};

pub fn make_simulation<'a>() -> Simulation<'a> {
    let forces = vec![&GRAVITY];

    // let mesh = {
    //     static POINT_SPACING_METERS: f32 = 3.0;
    //     static NUM_POINTS: usize = 8;

    //     let mut points: Vec<Point> = vec![];

    //     let mut x: f32 = 0.0;
    //     let mut y: f32 = 16.0;

    //     for i in 0..NUM_POINTS {
    //         if i % 2 == 0 {
    //             y -= POINT_SPACING_METERS;
    //         } else {
    //             x += POINT_SPACING_METERS;
    //         };

    //         points.push(Point {
    //             is_static: if i == 0 { true } else { false },
    //             // mass: 2.5,
    //             position: Vec3 {
    //                 x,
    //                 y,
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         })
    //     }

    //     let mut struts: Vec<(usize, usize)> = vec![];

    //     for i in 0..NUM_POINTS - 1 {
    //         struts.push((i, i + 1));
    //     }

    //     if NUM_POINTS > 2 {
    //         let i = NUM_POINTS - 3;
    //         let j = NUM_POINTS - 1;

    //         struts.push((i, j));
    //     }

    //     SpringyMesh::new(points, struts)
    // };

    let meshes = vec![
        SpringyMesh::new_box(
            Vec3 {
                x: -15.0,
                y: 10.0,
                ..Default::default()
            },
            10.0,
        ),
        SpringyMesh::new_box(
            Vec3 {
                x: 0.0,
                y: 10.0,
                ..Default::default()
            },
            10.0,
        ),
        SpringyMesh::new_box(
            Vec3 {
                x: 15.0,
                y: 10.0,
                ..Default::default()
            },
            10.0,
        ),
    ];

    let ground_plane_y: f32 = -10.0;
    let ground_plane_width: f32 = 60.0;
    let ground_plane_width_over_2: f32 = ground_plane_width / 2.0;

    Simulation {
        forces,
        wind: Default::default(),
        colliders: vec![LineSegmentCollider::new(
            Vec3 {
                x: -ground_plane_width_over_2,
                y: ground_plane_y,
                z: 0.0,
            },
            Vec3 {
                x: ground_plane_width_over_2,
                y: ground_plane_y,
                z: 0.0,
            },
        )],
        meshes,
    }
}
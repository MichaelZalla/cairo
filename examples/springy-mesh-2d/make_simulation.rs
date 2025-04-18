use cairo::{
    physics::simulation::{force::gravity::GRAVITY_POINT_FORCE, particle::Particle},
    vec::vec3::Vec3,
};

use crate::{
    simulation::Simulation, springy_mesh::SpringyMesh,
    static_line_segment_collider::StaticLineSegmentCollider,
};

pub fn make_simulation<'a>() -> Simulation<'a> {
    let forces = vec![&GRAVITY_POINT_FORCE];

    let springy_chain_mesh = {
        static POINT_SPACING_METERS: f32 = 3.0;
        static NUM_POINTS: usize = 8;

        let mut points: Vec<Particle> = vec![];

        let mut x: f32 = 0.0;
        let mut y: f32 = 16.0;

        for i in 0..NUM_POINTS {
            if i % 2 == 0 {
                y -= POINT_SPACING_METERS;
            } else {
                x += POINT_SPACING_METERS;
            };

            points.push(Particle {
                mass: if i == 0 { f32::INFINITY } else { 2.5 },
                position: Vec3 { x, y, z: 0.0 },
                ..Default::default()
            })
        }

        let mut struts: Vec<(usize, usize, bool)> = vec![];

        for i in 0..NUM_POINTS - 1 {
            struts.push((i, i + 1, false));
        }

        if NUM_POINTS > 2 {
            let i = NUM_POINTS - 3;
            let j = NUM_POINTS - 1;

            struts.push((i, j, false));
        }

        SpringyMesh::new(points, struts)
    };

    let meshes = vec![
        // SpringyMesh::new_box(
        //     Vec3 {
        //         x: -15.0,
        //         y: 10.0,
        //         ..Default::default()
        //     },
        //     10.0,
        // ),
        // SpringyMesh::new_box(
        //     Vec3 {
        //         x: 0.0,
        //         y: 10.0,
        //         ..Default::default()
        //     },
        //     10.0,
        // ),
        // SpringyMesh::new_box(
        //     Vec3 {
        //         x: 15.0,
        //         y: 10.0,
        //         ..Default::default()
        //     },
        //     10.0,
        // ),
        springy_chain_mesh,
    ];

    let ground_plane_y: f32 = -10.0;
    let ground_plane_width: f32 = 60.0;
    let ground_plane_width_over_2: f32 = ground_plane_width / 2.0;

    Simulation {
        forces,
        wind: Default::default(),
        static_colliders: vec![StaticLineSegmentCollider::new(
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

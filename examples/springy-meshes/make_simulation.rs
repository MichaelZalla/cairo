use sdl2::sys::SDL_STANDARD_GRAVITY;

use cairo::vec::vec3::Vec3;

use crate::force::{Force, Newtons};
use crate::point::Point;
use crate::simulation::Simulation;
use crate::springy_mesh::SpringyMesh;
use crate::state_vector::StateVector;
use crate::strut::Strut;

static GRAVITY: Force = |_state: &StateVector, _i: usize, _current_time: f32| -> Newtons {
    Vec3 {
        x: 0.0,
        y: -(SDL_STANDARD_GRAVITY as f32),
        z: 0.0,
    }
};

pub fn make_simulation<'a>() -> Simulation<'a> {
    let forces = vec![&GRAVITY];

    static POINT_SPACING_METERS: f32 = 3.0;
    static STRENGTH_PER_UNIT_LENGTH: f32 = 100.0;
    static DAMPER_PER_UNIT_LENGTH: f32 = 100.0;

    let mesh = {
        static NUM_POINTS: usize = 8;

        let mut points: Vec<Point> = vec![];

        let mut x: f32 = 0.0;
        let mut y: f32 = 16.0;

        for i in 0..NUM_POINTS {
            if i % 2 == 0 {
                y -= POINT_SPACING_METERS;
            } else {
                x += POINT_SPACING_METERS;
            };

            points.push(Point {
                position: Vec3 {
                    x,
                    y,
                    ..Default::default()
                },
                ..Default::default()
            })
        }

        let mut struts: Vec<Strut> = vec![];

        for i in 0..NUM_POINTS - 1 {
            let rest_length = (points[i + 1].position - points[i].position).mag();

            struts.push(Strut {
                points: (i, i + 1),
                rest_length,
                strength: STRENGTH_PER_UNIT_LENGTH / (rest_length / 1.0),
                damper: DAMPER_PER_UNIT_LENGTH / (rest_length / 1.0),
            })
        }

        if NUM_POINTS > 2 {
            let i = NUM_POINTS - 3;
            let j = NUM_POINTS - 1;

            let rest_length = (points[j].position - points[i].position).mag();

            struts.push(Strut {
                points: (i, j),
                rest_length,
                strength: STRENGTH_PER_UNIT_LENGTH / (rest_length / 1.0),
                damper: DAMPER_PER_UNIT_LENGTH / (rest_length / 1.0),
            })
        }

        SpringyMesh { points, struts }
    };

    Simulation {
        forces,
        wind: Default::default(),
        mesh,
    }
}

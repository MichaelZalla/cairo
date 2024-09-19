use cairo::vec::vec3::{self, Vec3};

use crate::{quaternion::Quaternion, rigid_body::RigidBody};

pub struct Simulation {
    pub rigid_bodies: Vec<RigidBody>,
}

impl Simulation {
    pub fn tick(&mut self, current_time: f32, cursor_world_space: Vec3) {
        for body in self.rigid_bodies.iter_mut() {
            let position = *body.transform.translation();

            if cursor_world_space.x == position.x && cursor_world_space.y == position.y {
                continue;
            }

            let body_to_cursor = cursor_world_space - position;

            let local_body_cursor_theta = body_to_cursor.as_normal().dot(vec3::RIGHT).acos();

            body.transform.set_translation(Vec3 {
                x: current_time.cos() * 5.0,
                y: current_time.sin() * 5.0,
                z: 0.0,
            });

            body.transform.set_orientation(Quaternion::new_2d(
                if cursor_world_space.y < position.y {
                    -local_body_cursor_theta
                } else {
                    local_body_cursor_theta
                },
            ));
        }
    }
}

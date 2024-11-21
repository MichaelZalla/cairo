use core::f32;

use crate::vec::vec3::{self, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub one_over_direction: Vec3,
    pub t: f32,
    pub triangle: Option<usize>,
}

fn get_one_over_direction_safe(d: Vec3) -> Vec3 {
    Vec3 {
        x: 1.0 / if d.x == 0.0 { f32::EPSILON } else { d.x },
        y: 1.0 / if d.y == 0.0 { f32::EPSILON } else { d.y },
        z: 1.0 / if d.z == 0.0 { f32::EPSILON } else { d.z },
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            direction: vec3::FORWARD,
            one_over_direction: get_one_over_direction_safe(vec3::FORWARD),
            t: f32::MAX,
            triangle: None,
        }
    }
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            one_over_direction: get_one_over_direction_safe(direction),
            ..Default::default()
        }
    }
}

pub fn grid(rows: usize, columns: usize, size: f32) -> Vec<Ray> {
    let mut rays = vec![Ray::new(Default::default(), -vec3::UP); rows * columns];

    let grid_left = -size / 2.0;

    let grid_near = grid_left;

    let ray_grid_column_alpha_step = 1.0 / columns as f32;

    let ray_grid_row_alpha_step = 1.0 / rows as f32;

    for z_offset in 0..columns {
        let z_alpha = z_offset as f32 * ray_grid_column_alpha_step;

        for x_offset in 0..rows {
            let x_alpha = x_offset as f32 * ray_grid_row_alpha_step;

            let ray = &mut rays[z_offset * columns + x_offset];

            ray.origin = Vec3 {
                x: grid_left + size * (0.5 * ray_grid_row_alpha_step + x_alpha),
                z: grid_near + size * (0.5 * ray_grid_column_alpha_step + z_alpha),
                ..Default::default()
            };
        }
    }

    rays
}

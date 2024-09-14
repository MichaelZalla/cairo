use cairo::{buffer::Buffer2D, color, graphics::Graphics, vec::vec3::Vec3};

use crate::{
    coordinates::world_to_screen_space,
    renderable::Renderable,
    state_vector::{FromStateVector, StateVector, ToStateVector},
};

#[derive(Default, Debug, Copy, Clone)]
pub struct Point {
    pub is_static: bool,
    pub position: Vec3,
    pub velocity: Vec3,
}

pub static POINT_MASS: f32 = 2.5;

impl ToStateVector for Point {
    fn write_to(&self, state: &mut StateVector, n: usize, i: usize) {
        state.data[i] = self.position;
        state.data[i + n] = self.velocity;
    }
}

impl FromStateVector for Point {
    fn write_from(&mut self, state: &StateVector, n: usize, i: usize) {
        self.velocity = state.data[i + n];
        self.position = state.data[i];
    }
}

impl Renderable for Point {
    fn render(&self, buffer: &mut Buffer2D, buffer_center: &Vec3) {
        static POINT_SIZE: u32 = 8;
        static POINT_SIZE_OVER_2: u32 = POINT_SIZE / 2;

        let world_space_position = self.position;

        let screen_space_position = world_to_screen_space(&world_space_position, buffer_center);

        let x = screen_space_position.x as i32;
        let y = screen_space_position.y as i32;

        if let Some((x, y, width, height)) = Graphics::clip_rectangle(
            x - POINT_SIZE_OVER_2 as i32,
            y - POINT_SIZE_OVER_2 as i32,
            POINT_SIZE,
            POINT_SIZE,
            buffer,
        ) {
            Graphics::rectangle(buffer, x, y, width, height, Some(&color::YELLOW), None)
        }
    }
}

use sdl2::keyboard::Keycode;

use cairo::{
    buffer::Buffer2D,
    color,
    device::{keyboard::KeyboardState, mouse::MouseState},
    graphics::Graphics,
};

pub struct Graph {
    screen_origin: (i32, i32),
    pixels_per_unit: u32,
}

impl Graph {
    pub fn new(screen_origin: (i32, i32), pixels_per_unit: u32) -> Self {
        Self {
            screen_origin,
            pixels_per_unit,
        }
    }

    fn cartesian_to_screen(&self, x: f32, y: f32) -> (i32, i32) {
        (
            self.screen_origin.0 + (x * self.pixels_per_unit as f32) as i32,
            self.screen_origin.1 + (-y * self.pixels_per_unit as f32) as i32,
        )
    }

    fn screen_to_cartesian(&self, x: i32, y: i32) -> (f32, f32) {
        (
            (x as f32 - self.screen_origin.0 as f32) / self.pixels_per_unit as f32,
            -(y as f32 - self.screen_origin.1 as f32) / self.pixels_per_unit as f32,
        )
    }

    pub fn update(&mut self, keyboard_state: &mut KeyboardState, mouse_state: &mut MouseState) {
        // Translate screen origin with W-A-S-D.

        static TRANSLATION_SPEED: i32 = 8;

        if keyboard_state.keys_pressed.contains(&Keycode::W) {
            self.screen_origin.1 += TRANSLATION_SPEED;
        } else if keyboard_state.keys_pressed.contains(&Keycode::A) {
            self.screen_origin.0 += TRANSLATION_SPEED;
        } else if keyboard_state.keys_pressed.contains(&Keycode::S) {
            self.screen_origin.1 -= TRANSLATION_SPEED;
        } else if keyboard_state.keys_pressed.contains(&Keycode::D) {
            self.screen_origin.0 -= TRANSLATION_SPEED;
        }

        // Change zoom level with scroll wheel.

        if let Some(event) = &mouse_state.wheel_event {
            if event.delta > 0 {
                self.pixels_per_unit = (self.pixels_per_unit + 1).clamp(2, 64);
            } else {
                self.pixels_per_unit = (self.pixels_per_unit as i32 - 1).clamp(2, 64) as u32;
            }
        }
    }

    pub fn render(&self, buffer: &mut Buffer2D) {
        let screen_origin = self.screen_origin;
        let pixels_per_unit = self.pixels_per_unit;

        // Draw the X axis.

        let (x1, y1, x2, y2) = (0, screen_origin.1, buffer.width as i32, screen_origin.1);

        Graphics::line(buffer, x1, y1, x2, y2, &color::YELLOW);

        // Draw the Y axis.

        let (x1, y1, x2, y2) = (
            screen_origin.0,
            0,
            screen_origin.0,
            (buffer.height - 1) as i32,
        );

        Graphics::line(buffer, x1, y1, x2, y2, &color::YELLOW);

        // Plot ticks.

        static UNITS_PER_TICK: u32 = 4;

        let screen_origin_cartesian = (0.0 as f32, 0.0 as f32);

        let screen_top_cartesian = self.screen_to_cartesian((buffer.width / 2) as i32, 0);

        let screen_bottom_cartesian =
            self.screen_to_cartesian((buffer.width / 2) as i32, (buffer.height - 1) as i32);

        let screen_left_cartesian = self.screen_to_cartesian(0, screen_origin.1);

        let screen_right_cartesian =
            self.screen_to_cartesian((buffer.width - 1) as i32, screen_origin.1);

        static TICK_WIDTH_PIXELS: f32 = 3.0;

        let tick_width_cartesian = TICK_WIDTH_PIXELS / pixels_per_unit as f32;

        // Plot vertical ticks.

        let units_above_origin = (screen_top_cartesian.1 - screen_origin_cartesian.1) as i32;
        let units_below_origin = (screen_origin_cartesian.1 - screen_bottom_cartesian.1) as i32;

        let y_start_cartesian = -units_below_origin + (units_below_origin % UNITS_PER_TICK as i32);
        let y_end_cartesian = units_above_origin;

        for tick_y in (y_start_cartesian..y_end_cartesian).step_by(UNITS_PER_TICK as usize) {
            let tick_start = self.cartesian_to_screen(-tick_width_cartesian, tick_y as f32);
            let tick_end = self.cartesian_to_screen(tick_width_cartesian, tick_y as f32);

            Graphics::line(
                buffer,
                tick_start.0,
                tick_start.1,
                tick_end.0,
                tick_end.1,
                &color::YELLOW,
            )
        }

        // Plot horizontal ticks.

        let units_left_of_origin = (screen_origin_cartesian.0 - screen_left_cartesian.0) as i32;
        let units_right_of_origin = (screen_right_cartesian.0 - screen_origin_cartesian.0) as i32;

        let x_start_cartesian =
            -units_left_of_origin + (units_left_of_origin % UNITS_PER_TICK as i32);

        let x_end_cartesian = units_right_of_origin;

        for tick_x in (x_start_cartesian..x_end_cartesian).step_by(UNITS_PER_TICK as usize) {
            let tick_start = self.cartesian_to_screen(tick_x as f32, tick_width_cartesian);
            let tick_end = self.cartesian_to_screen(tick_x as f32, -tick_width_cartesian);

            Graphics::line(
                buffer,
                tick_start.0,
                tick_start.1,
                tick_end.0,
                tick_end.1,
                &color::YELLOW,
            )
        }
    }
}

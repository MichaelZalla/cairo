use sdl2::keyboard::Keycode;

use cairo::{
    buffer::Buffer2D,
    color::{self, Color},
    device::{keyboard::KeyboardState, mouse::MouseState},
    graphics::Graphics,
};

pub type GraphingFunction = dyn Fn(f32) -> f32;
pub type BoxedGraphingFunction = Box<GraphingFunction>;

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

        for keycode in &keyboard_state.pressed_keycodes {
            match keycode {
                &Keycode::W => {
                    self.screen_origin.1 += TRANSLATION_SPEED;
                }
                &Keycode::A => {
                    self.screen_origin.0 += TRANSLATION_SPEED;
                }
                &Keycode::S => {
                    self.screen_origin.1 -= TRANSLATION_SPEED;
                }
                &Keycode::D => {
                    self.screen_origin.0 -= TRANSLATION_SPEED;
                }
                _ => (),
            }
        }

        // Change zoom level with scroll wheel.

        if let Some(event) = &mouse_state.wheel_event {
            let delta: i32 = if event.delta > 0 { 1 } else { -1 };

            self.pixels_per_unit = (self.pixels_per_unit as i32 + delta).clamp(2, 256) as u32;
        }
    }

    pub fn axes(&self, buffer: &mut Buffer2D) {
        self.render_axes(buffer);
        self.render_ticks(buffer);
    }

    fn render_axes(&self, buffer: &mut Buffer2D) {
        let screen_origin = self.screen_origin;

        let color_u32 = color::DARK_GRAY.to_u32();

        // Draw the X axis.

        let (x1, y1, x2, y2) = (0, screen_origin.1, buffer.width as i32, screen_origin.1);

        Graphics::line(buffer, x1, y1, x2, y2, color_u32);

        // Draw the Y axis.

        let (x1, y1, x2, y2) = (
            screen_origin.0,
            0,
            screen_origin.0,
            (buffer.height - 1) as i32,
        );

        Graphics::line(buffer, x1, y1, x2, y2, color_u32);
    }

    fn render_ticks(&self, buffer: &mut Buffer2D) {
        let screen_origin = self.screen_origin;
        let pixels_per_unit = self.pixels_per_unit;

        // Tick frequency.

        static UNITS_PER_TICK: u32 = 1;

        // Screen coordinates to cartesian.

        let screen_origin_cartesian = (0.0_f32, 0.0_f32);

        let screen_top_cartesian = self.screen_to_cartesian((buffer.width / 2) as i32, 0);

        let screen_bottom_cartesian =
            self.screen_to_cartesian((buffer.width / 2) as i32, (buffer.height - 1) as i32);

        let screen_left_cartesian = self.screen_to_cartesian(0, screen_origin.1);

        let screen_right_cartesian =
            self.screen_to_cartesian((buffer.width - 1) as i32, screen_origin.1);

        // Compute the pixel-constant tick width, as a cartesian distance.

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
                color::DARK_GRAY.to_u32(),
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
                color::DARK_GRAY.to_u32(),
            )
        }
    }

    #[allow(unused)]
    pub fn functions(
        &self,
        functions: &Vec<(BoxedGraphingFunction, Color)>,
        buffer: &mut Buffer2D,
    ) {
        for (function, color) in functions {
            self.function(function, *color, buffer);
        }
    }

    pub fn function(&self, function: &GraphingFunction, color: Color, buffer: &mut Buffer2D) {
        for i in 0..buffer.width - 2 {
            let (x1_cartesian, _) = self.screen_to_cartesian(i as i32, 0);
            let y1_cartesian = function(x1_cartesian);

            let (x2_cartesian, _) = self.screen_to_cartesian(i as i32 + 1, 0);
            let y2_cartesian = function(x2_cartesian);

            self.line(
                x1_cartesian,
                y1_cartesian,
                x2_cartesian,
                y2_cartesian,
                color,
                buffer,
            );
        }
    }

    #[allow(unused)]
    pub fn point(&self, x: f32, y: f32, color: Color, buffer: &mut Buffer2D) {
        let (screen_x, screen_y) = self.cartesian_to_screen(x, y);

        if screen_x >= 0
            && screen_x < buffer.width as i32
            && screen_y >= 0
            && screen_y < buffer.height as i32
        {
            buffer.set(screen_x as u32, screen_y as u32, color.to_u32());
        }
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, buffer: &mut Buffer2D) {
        let start = self.cartesian_to_screen(x1, y1);
        let end = self.cartesian_to_screen(x2, y2);

        Graphics::line(buffer, start.0, start.1, end.0, end.1, color.to_u32());
    }
}

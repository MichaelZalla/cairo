use std::collections::HashSet;

use rand::Rng;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::BlendMode;

use crate::debug_print;
use crate::device::{
    GameController, GameControllerState, KeyboardState, MouseEvent, MouseEventKind, MouseState,
};

use crate::context::{get_application_context, get_backbuffer, ApplicationContext};

pub struct App {
    pub context: ApplicationContext,
    pub aspect_ratio: f32,
}

impl App {
    pub fn new(window_title: &str, window_width: u32, aspect_ratio: f32) -> Self {
        let window_height: u32 = (window_width as f32 / aspect_ratio) as u32;

        let context = get_application_context(
            window_title,
            window_width,
            window_height,
            false,
            false,
            true,
        )
        .unwrap();

        return App {
            context,
            aspect_ratio,
        };
    }

    pub fn run<U, R>(mut self, update: &mut U, render: &mut R) -> Result<(), String>
    where
        U: FnMut(&KeyboardState, &MouseState, &GameControllerState, f32) -> (),
        R: FnMut() -> Result<Vec<u32>, String>,
    {
        let texture_creator = self.context.rendering_context.canvas.texture_creator();

        let mut backbuffer = get_backbuffer(
            &self.context.rendering_context,
            &texture_creator,
            BlendMode::None,
        )
        .unwrap();

        // Set up scene here!

        let ticks_per_second = self.context.timer.performance_frequency();

        let frame_rate_limit = 120;

        let desired_ticks_per_frame: u64 = ticks_per_second / frame_rate_limit;

        let mut frame_start_tick: u64 = self.context.timer.performance_counter();
        let mut frame_end_tick: u64;

        let mut rng = rand::thread_rng();

        let mut prev_mouse_clicks = HashSet::new();

        let mut prev_game_controller_state: GameControllerState = GameController::new().state;

        // Main event loop

        'main: loop {
            // Main loop

            let now_tick = self.context.timer.performance_counter();

            let ticks_slept = now_tick - frame_start_tick;

            let seconds_slept: f32 = ticks_slept as f32 / ticks_per_second as f32;

            let milliseconds_slept = seconds_slept * 1000.0;

            debug_print!(
                "Slept for {} ticks, {}s, {}ms!",
                ticks_slept,
                seconds_slept,
                milliseconds_slept
            );

            // Event polling

            let events = self.context.events.poll_iter();

            let mut mouse_state = MouseState::new();

            let mut keyboard_state = KeyboardState::new();

            let mut game_controller = GameController::new();

            let controller = self.context.game_controllers[0].as_ref();

            if controller.is_some() {
                let unwrapped = controller.unwrap();

                game_controller.id = unwrapped.id;
                game_controller.name = unwrapped.name.clone();
                game_controller.state = prev_game_controller_state;
            }

            for event in events {
                match event {
                    Event::Quit { .. } => break 'main,

                    // Event::MouseMotion { x, y, .. } => {
                    //     last_known_mouse_x = x;
                    //     last_known_mouse_y = y;
                    // }
                    Event::MouseWheel { direction, y, .. } => {
                        mouse_state.wheel_did_move = true;
                        mouse_state.wheel_direction = direction;
                        mouse_state.wheel_y = y;
                    }

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => match keycode {
                        Keycode::Escape { .. } => break 'main,
                        _ => {
                            keyboard_state.keys_pressed.push(keycode);
                        }
                    },

                    Event::ControllerDeviceAdded { which, .. } => {
                        println!("Connected controller {}", which);
                    }

                    Event::ControllerDeviceRemoved { which, .. } => {
                        println!("Disconnected controller {}", which);
                    }

                    Event::JoyButtonDown { button_idx, .. } => {
                        println!("Button down! {}", button_idx);
                    }

                    Event::JoyButtonUp { button_idx, .. } => {
                        println!("Button up! {}", button_idx);
                    }

                    Event::ControllerButtonDown { button, .. } => {
                        game_controller.set_button_state(button, true);
                    }

                    Event::ControllerButtonUp { button, .. } => {
                        game_controller.set_button_state(button, false);
                    }

                    Event::ControllerAxisMotion { axis, value, .. } => {
                        game_controller.set_joystick_state(axis, value);
                    }

                    _ => {}
                }
            }

            // Read the current mouse state

            let current_mouse_state = self.context.events.mouse_state();

            // Read any mouse click signals

            let mouse_clicks = current_mouse_state.pressed_mouse_buttons().collect();

            // Get the difference between the old and new signals

            let old_mouse_clicks = &prev_mouse_clicks - &mouse_clicks;
            let new_mouse_clicks = &mouse_clicks - &prev_mouse_clicks;

            // Use the difference to construct any button-click event(s)

            if !new_mouse_clicks.is_empty() || !old_mouse_clicks.is_empty() {
                let mut is_down: bool = false;

                let source = if !new_mouse_clicks.is_empty() {
                    is_down = true;
                    new_mouse_clicks
                } else {
                    old_mouse_clicks
                };

                let button: MouseButton = source
                    .into_iter()
                    .collect::<Vec<MouseButton>>()
                    .first()
                    .unwrap()
                    .to_owned();

                match button {
                    MouseButton::Left | MouseButton::Right | MouseButton::Middle => {
                        mouse_state.button_event = Some(MouseEvent {
                            button,
                            kind: if is_down {
                                MouseEventKind::Down
                            } else {
                                MouseEventKind::Up
                            },
                        })
                    }
                    _ => {
                        // Do nothing?
                    }
                }
            }

            prev_mouse_clicks = mouse_clicks;

            prev_game_controller_state = game_controller.state.clone();

            // Cache input device states

            mouse_state.position.0 = current_mouse_state.x();
            mouse_state.position.1 = current_mouse_state.y();

            // Update current scene

            update(
                &keyboard_state,
                &mouse_state,
                &game_controller.state,
                seconds_slept,
            );

            // Render current scene to backbuffer

            backbuffer
                .with_lock(None, |write_only_byte_array, _pitch| {
                    // Render current scene

                    match render() {
                        Ok(pixels_as_u32_slice) => {
                            let pixels_as_u8_slice: &[u8] =
                                bytemuck::cast_slice(&pixels_as_u32_slice);

                            let mut index = 0;

                            while index < pixels_as_u8_slice.len() {
                                write_only_byte_array[index] = pixels_as_u8_slice[index];
                                index += 1;
                            }
                        }
                        Err(_e) => {
                            // Do nothing?
                        }
                    }
                })
                .unwrap();

            // Flip buffers

            self.context
                .rendering_context
                .canvas
                .copy(&backbuffer, None, None)?;

            self.context.rendering_context.canvas.present();

            frame_end_tick = self.context.timer.performance_counter();

            // Report framerate

            let ticks_for_current_frame = frame_end_tick - frame_start_tick;

            // let frames_per_second = ticks_for_current_frame as f64 / ticks_per_second as f64;

            let frames_per_second = ticks_per_second / ticks_for_current_frame;

            let unused_ticks: u64;

            if ticks_for_current_frame < desired_ticks_per_frame {
                unused_ticks = std::cmp::min(
                    desired_ticks_per_frame,
                    desired_ticks_per_frame - ticks_for_current_frame,
                );
            } else {
                unused_ticks = 0;
            }

            let unused_seconds = (unused_ticks as f64 / ticks_per_second as f64) as f64;
            let unused_milliseconds = unused_seconds * 1000.0;

            let random: u32 = rng.gen();
            let modulo: u32 = 30;

            if random % modulo == 0 {
                debug_print!("===========================");
                debug_print!("ticks_per_second={}", ticks_per_second);
                debug_print!("frame_start_tick={}", frame_start_tick);
                debug_print!("frame_end_tick={}", frame_end_tick);
                debug_print!("desired_ticks_per_frame={}", desired_ticks_per_frame);
                debug_print!("ticks_for_current_frame={}", ticks_for_current_frame);
                debug_print!("unused_ticks={}", unused_ticks);
                debug_print!("frames_per_second={}", frames_per_second);
                debug_print!("unused_seconds={}", unused_seconds);
                debug_print!("unused_milliseconds={}", unused_milliseconds);
            }

            frame_start_tick = self.context.timer.performance_counter();

            // Sleep if we can...

            self.context.timer.delay(unused_milliseconds.floor() as u32);
        }

        Ok(())
    }
}

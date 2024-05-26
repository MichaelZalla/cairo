use std::collections::HashSet;
use std::ptr;

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::{event::Event, render::Texture};

use crate::{
    debug_print,
    device::{
        GameController, GameControllerState, KeyboardState, MouseEvent, MouseEventKind, MouseState,
    },
    time::TimingInfo,
};

use self::context::{make_application_context, make_backbuffer, ApplicationContext};
use self::resolution::{Resolution, DEFAULT_WINDOW_RESOLUTION};

pub mod context;
pub mod resolution;

#[derive(Debug, Clone)]
pub struct AppWindowInfo {
    pub title: String,
    pub canvas_resolution: Resolution,
    pub window_resolution: Resolution,
    pub full_screen: bool,
    pub show_cursor: bool,
    pub relative_mouse_mode: bool,
    pub vertical_sync: bool,
}

impl Default for AppWindowInfo {
    fn default() -> Self {
        Self {
            title: "App".to_string(),
            window_resolution: DEFAULT_WINDOW_RESOLUTION,
            canvas_resolution: DEFAULT_WINDOW_RESOLUTION,
            full_screen: false,
            show_cursor: true,
            relative_mouse_mode: false,
            vertical_sync: false,
        }
    }
}

pub struct App {
    pub window_info: AppWindowInfo,
    pub context: ApplicationContext,
    pub backbuffer: Texture,
    pub timing_info: TimingInfo,
}

impl App {
    pub fn new(window_info: &mut AppWindowInfo) -> Self {
        let context = make_application_context(window_info).unwrap();

        let timing_info: TimingInfo = Default::default();

        window_info.window_resolution = Resolution {
            width: context.screen_width,
            height: context.screen_height,
        };

        let app_window_info = window_info.clone();

        let texture_creator = context.rendering_context.canvas.borrow().texture_creator();

        let backbuffer =
            make_backbuffer(window_info.canvas_resolution, &texture_creator, None).unwrap();

        App {
            window_info: app_window_info,
            context,
            backbuffer,
            timing_info,
        }
    }

    pub fn resize_window(&mut self, new_resolution: Resolution) -> Result<(), String> {
        let mut canvas = self.context.rendering_context.canvas.borrow_mut();

        match canvas
            .window_mut()
            .set_size(new_resolution.width, new_resolution.height)
        {
            Ok(_) => {
                // Update window info.

                self.window_info.window_resolution = new_resolution;

                Ok(())
            }
            Err(e) => Err(format!("Failed to resize app window: {}", e)),
        }
    }

    pub fn resize_canvas(&mut self, new_resolution: Resolution) -> Result<(), String> {
        let canvas = self.context.rendering_context.canvas.borrow_mut();

        // Re-allocates a backbuffer.

        let texture_creator = canvas.texture_creator();

        match make_backbuffer(new_resolution, &texture_creator, None) {
            Ok(texture) => {
                self.backbuffer = texture;

                self.window_info.canvas_resolution = new_resolution;

                Ok(())
            }
            Err(e) => Err(format!(
                "Failed to reallocate backbuffer in App::resize_canvas(): {}",
                e
            )),
        }
    }

    pub fn run<U, R>(mut self, update: &mut U, render: &mut R) -> Result<(), String>
    where
        U: FnMut(
            &mut Self,
            &mut KeyboardState,
            &mut MouseState,
            &mut GameControllerState,
        ) -> Result<(), String>,
        R: FnMut(u32) -> Result<Vec<u32>, String>,
    {
        let timer_subsystem = self.context.sdl_context.timer()?;

        let ticks_per_second = timer_subsystem.performance_frequency();

        let frame_rate_limit = 120;

        let desired_ticks_per_frame: u64 = ticks_per_second / frame_rate_limit;

        let mut frame_start: u64 = timer_subsystem.performance_counter();
        let mut frame_end: u64;

        let mut prev_mouse_buttons_down = HashSet::new();

        let mut prev_game_controller_state: GameControllerState = GameController::new().state;

        let mut frames_rendered: u32 = 0;

        let mut last_update_tick = timer_subsystem.performance_counter();

        // Main event loop

        'main: loop {
            // Main loop

            let now = timer_subsystem.performance_counter();

            let ticks_slept = now - frame_start;

            let seconds_slept: f32 = ticks_slept as f32 / ticks_per_second as f32;

            self.timing_info.milliseconds_slept = seconds_slept * 1000.0;

            debug_print!(
                "Slept for {} ticks, {}s, {}ms!",
                ticks_slept,
                seconds_slept,
                self.timing_info.milliseconds_slept
            );

            // Event polling

            let mut event_pump = self.context.sdl_context.event_pump()?;

            let events = event_pump.poll_iter();

            let mut mouse_state: MouseState = Default::default();

            let mut keyboard_state: KeyboardState = Default::default();

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

                    Event::MouseMotion { xrel, yrel, .. } => {
                        mouse_state.relative_motion.0 = xrel;
                        mouse_state.relative_motion.1 = yrel;
                    }

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

            let current_mouse_state = event_pump.mouse_state();

            // Read any mouse click signals

            let mouse_buttons_down: HashSet<MouseButton> =
                current_mouse_state.pressed_mouse_buttons().collect();

            mouse_state.buttons_down = mouse_buttons_down.clone();

            // Get the difference between the old and new signals

            let old_mouse_clicks = &prev_mouse_buttons_down - &mouse_buttons_down;
            let new_mouse_clicks = &mouse_buttons_down - &prev_mouse_buttons_down;

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

            prev_mouse_buttons_down = mouse_buttons_down;

            prev_game_controller_state = game_controller.state.clone();

            // Cache input device states

            mouse_state.prev_position = mouse_state.position;
            mouse_state.prev_ndc_position = mouse_state.ndc_position;

            mouse_state.position.0 = current_mouse_state.x();
            mouse_state.position.1 = current_mouse_state.y();

            mouse_state.ndc_position.0 =
                mouse_state.position.0 as f32 / self.window_info.window_resolution.width as f32;
            mouse_state.ndc_position.1 =
                mouse_state.position.1 as f32 / self.window_info.window_resolution.height as f32;

            // Update current scene

            let ticks_since_last_update: u64 =
                timer_subsystem.performance_counter() - last_update_tick;

            self.timing_info.seconds_since_last_update =
                ticks_since_last_update as f32 / ticks_per_second as f32;

            self.timing_info.uptime_seconds += self.timing_info.seconds_since_last_update;

            update(
                &mut self,
                &mut keyboard_state,
                &mut mouse_state,
                &mut game_controller.state,
            )?;

            last_update_tick = timer_subsystem.performance_counter();

            // Render current scene to backbuffer

            let cw = &mut self.context.rendering_context.canvas.borrow_mut();

            self.backbuffer
                .with_lock(None, |write_only_byte_array, _pitch| {
                    // Render current scene

                    match render(self.timing_info.current_frame_index) {
                        Ok(pixels_as_u32_slice) => unsafe {
                            let pixels_as_u8_slice: &[u8] = &*(ptr::slice_from_raw_parts(
                                pixels_as_u32_slice.as_ptr() as *const u8,
                                pixels_as_u32_slice.len() * 4,
                            ));

                            let mut index = 0;

                            while index < pixels_as_u8_slice.len() {
                                write_only_byte_array[index] = pixels_as_u8_slice[index];
                                index += 1;
                            }
                        },
                        Err(_e) => {
                            // Do nothing?
                        }
                    }
                })
                .unwrap();

            // Flip buffers

            // Note that Canvas<Window>::copy() will automatically stretch our
            // backbuffer to fit the current window size, if `dst` is `None`.

            cw.copy(&self.backbuffer, None, None)?;

            cw.present();

            frame_end = timer_subsystem.performance_counter();

            // Report framerate

            let ticks_for_current_frame = frame_end - frame_start;

            // let frames_per_second = ticks_for_current_frame as f64 / ticks_per_second as f64;

            self.timing_info.frames_per_second =
                (ticks_per_second as f64 / ticks_for_current_frame as f64) as f32;

            let unused_ticks = if ticks_for_current_frame < desired_ticks_per_frame {
                std::cmp::min(
                    desired_ticks_per_frame,
                    desired_ticks_per_frame - ticks_for_current_frame,
                )
            } else {
                0
            };

            self.timing_info.unused_seconds =
                (unused_ticks as f64 / ticks_per_second as f64) as f32;

            self.timing_info.unused_milliseconds = self.timing_info.unused_seconds * 1000.0;

            let unused_seconds = unused_ticks as f64 / ticks_per_second as f64;
            let _unused_milliseconds = unused_seconds * 1000.0;

            if frames_rendered % 50 == 0 {
                debug_print!("frames_per_second={}", self.timing_info.frames_per_second);
                debug_print!("unused_seconds={}", unused_seconds);
                debug_print!("unused_milliseconds={}", unused_milliseconds);
            }

            frame_start = timer_subsystem.performance_counter();

            // Sleep if we can...

            timer_subsystem.delay(self.timing_info.unused_milliseconds.floor() as u32);

            // @NOTE(mzalla) Will overflow, and that's okay.
            frames_rendered += 1;

            self.timing_info.current_frame_index = frames_rendered;
        }

        Ok(())
    }
}

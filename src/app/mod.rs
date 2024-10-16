use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use sdl2::event::{EventWatch, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::video::{FullscreenType, Window};
use sdl2::{event::Event, render::Texture};

use crate::{
    device::{
        game_controller::{GameController, GameControllerState},
        keyboard::KeyboardState,
        mouse::{MouseDragEvent, MouseEvent, MouseEventKind, MouseState, MouseWheelEvent},
    },
    stats::CycleCounters,
    time::TimingInfo,
};

use window::AppWindowingMode;

use context::{make_application_context, make_canvas_texture, ApplicationContext};
use profile::AppCycleCounter;
use resolution::{Resolution, DEFAULT_WINDOW_RESOLUTION};

mod profile;

pub mod context;
pub mod resolution;
pub mod window;

#[derive(Debug, Clone)]
pub struct AppWindowInfo {
    pub title: String,
    pub canvas_resolution: Resolution,
    pub window_resolution: Resolution,
    pub windowing_mode: AppWindowingMode,
    pub show_cursor: bool,
    pub relative_mouse_mode: bool,
    pub vertical_sync: bool,
    pub resizable: bool,
}

impl Default for AppWindowInfo {
    fn default() -> Self {
        Self {
            title: "App".to_string(),
            window_resolution: DEFAULT_WINDOW_RESOLUTION,
            canvas_resolution: DEFAULT_WINDOW_RESOLUTION,
            show_cursor: true,
            windowing_mode: Default::default(),
            relative_mouse_mode: false,
            vertical_sync: false,
            resizable: false,
        }
    }
}

pub struct App {
    pub window_info: Rc<RefCell<AppWindowInfo>>,
    pub is_resizing_self: Rc<RefCell<bool>>,
    pub context: ApplicationContext,
    pub canvas_texture: Rc<RefCell<Texture>>,
    pub timing_info: TimingInfo,
    are_updates_paused: bool,
    #[cfg(feature = "debug_cycle_counts")]
    pub cycle_counters: CycleCounters,
}

impl App {
    pub fn new<'a>(
        window_info: &mut AppWindowInfo,
        rod: &'a impl Fn(Option<u32>, Option<Resolution>, &mut [u8]) -> Result<(), String>,
    ) -> (Self, Option<EventWatch<'a, impl Fn(Event) + 'a>>) {
        let context = make_application_context(window_info).unwrap();

        let timing_info: TimingInfo = Default::default();

        window_info.window_resolution = Resolution {
            width: context.screen_width,
            height: context.screen_height,
        };

        let window_info = window_info.clone();

        let canvas_window_rc = context.rendering_context.canvas.clone();

        let texture_creator = context.rendering_context.canvas.borrow().texture_creator();

        let canvas_texture =
            make_canvas_texture(window_info.canvas_resolution, &texture_creator, None).unwrap();

        let resizable = window_info.resizable;

        let window_info_rc = Rc::new(RefCell::new(window_info));
        let window_info_rc_clone = window_info_rc.clone();

        let canvas_texture_rc = Rc::new(RefCell::new(canvas_texture));
        let canvas_texture_rc_clone = canvas_texture_rc.clone();

        let is_resizing_self_rc = Rc::new(RefCell::new(false));
        let is_resizing_self_rc_clone = is_resizing_self_rc.clone();

        let event_watch = if resizable {
            let watch =
                context.event_subsystem.add_event_watch(move |event| {
                    if let Event::Window {
                        timestamp: _timestamp,
                        window_id: _window_id,
                        win_event:
                            WindowEvent::Resized(width, height)
                            | WindowEvent::SizeChanged(width, height),
                    } = event
                    {
                        let is_resizing_self = is_resizing_self_rc_clone.borrow();

                        if *is_resizing_self {
                            return;
                        }

                        let mut canvas_window = (*canvas_window_rc).borrow_mut();
                        let mut window_info = (*window_info_rc_clone).borrow_mut();
                        let mut canvas_texture = (*canvas_texture_rc_clone).borrow_mut();

                        let new_resolution = Resolution {
                            width: width as u32,
                            height: height as u32,
                        };

                        handle_window_resize_event(
                            &mut canvas_window,
                            &mut window_info,
                            &mut canvas_texture,
                            new_resolution,
                        )
                        .unwrap();

                        render_and_present(
                            &mut canvas_window,
                            &mut canvas_texture,
                            None,
                            None,
                            Some(new_resolution),
                            rod,
                        )
                        .unwrap();
                    };
                });

            Some(watch)
        } else {
            None
        };

        let app = App {
            window_info: window_info_rc,
            context,
            canvas_texture: canvas_texture_rc,
            is_resizing_self: is_resizing_self_rc,
            timing_info,
            are_updates_paused: false,
            #[cfg(feature = "debug_cycle_counts")]
            cycle_counters: Default::default(),
        };

        (app, event_watch)
    }

    pub fn pause_updates(&mut self) {
        self.are_updates_paused = true;
    }

    pub fn resume_updates(&mut self) {
        self.are_updates_paused = false;
    }

    pub fn toggle_updates(&mut self) {
        self.are_updates_paused = !self.are_updates_paused;
    }

    pub fn set_windowing_mode(&mut self, windowing_mode: AppWindowingMode) -> Result<(), String> {
        let mut canvas = self.context.rendering_context.canvas.borrow_mut();
        let mut window_info = self.window_info.borrow_mut();
        let mut canvas_texture = self.canvas_texture.borrow_mut();

        {
            *self.is_resizing_self.borrow_mut() = true;
        }

        handle_set_window_mode_event(
            &mut canvas,
            &mut window_info,
            &mut canvas_texture,
            windowing_mode,
        )?;

        if windowing_mode == AppWindowingMode::Windowed {
            handle_window_resize_event(
                &mut canvas,
                &mut window_info,
                &mut canvas_texture,
                DEFAULT_WINDOW_RESOLUTION,
            )?;
        }

        {
            *self.is_resizing_self.borrow_mut() = false;
        }

        Ok(())
    }

    pub fn resize_window(&mut self, resolution: Resolution) -> Result<(), String> {
        let mut canvas = self.context.rendering_context.canvas.borrow_mut();
        let mut window_info = self.window_info.borrow_mut();
        let mut canvas_texture = self.canvas_texture.borrow_mut();

        {
            *self.is_resizing_self.borrow_mut() = true;
        }

        handle_window_resize_event(
            &mut canvas,
            &mut window_info,
            &mut canvas_texture,
            resolution,
        )?;

        {
            *self.is_resizing_self.borrow_mut() = false;
        }

        Ok(())
    }

    pub fn run<U, R>(mut self, update: &mut U, render: &R) -> Result<(), String>
    where
        U: FnMut(
            &mut Self,
            &mut KeyboardState,
            &mut MouseState,
            &mut GameControllerState,
        ) -> Result<(), String>,
        R: Fn(Option<u32>, Option<Resolution>, &mut [u8]) -> Result<(), String>,
    {
        let timer_subsystem = self.context.sdl_context.timer()?;

        let ticks_per_second = timer_subsystem.performance_frequency();

        let frame_rate_limit = 120;

        let desired_ticks_per_frame: u64 = ticks_per_second / frame_rate_limit;

        let mut frame_start: u64 = timer_subsystem.performance_counter();
        let mut frame_end: u64;

        let mut prev_mouse_position = (0, 0);
        let mut prev_mouse_ndc_position = (0.0, 0.0);
        let mut prev_mouse_buttons_down = HashSet::new();

        let mut prev_game_controller_state: GameControllerState = GameController::new().state;

        let mut frames_rendered: u32 = 0;

        let mut last_update_tick = timer_subsystem.performance_counter();

        // Main event loop

        'main: loop {
            #[cfg(feature = "debug_cycle_counts")]
            {
                self.cycle_counters.reset();

                self.cycle_counters
                    .get_mut(AppCycleCounter::Run as usize)
                    .start();
            }

            // Main loop

            let now = timer_subsystem.performance_counter();

            let ticks_slept = now - frame_start;

            let seconds_slept: f32 = ticks_slept as f32 / ticks_per_second as f32;

            self.timing_info.milliseconds_slept = seconds_slept * 1000.0;

            let mut should_update_step_forward = false;

            // #[cfg(feature = "print_timing_info")]
            // println!(
            //     "Slept for {} ticks, {}s, {}ms!",
            //     ticks_slept, seconds_slept, self.timing_info.milliseconds_slept
            // );

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

                game_controller.name.clone_from(&unwrapped.name);

                game_controller.state = prev_game_controller_state;
            }

            for event in events {
                match event {
                    Event::Quit { .. } => break 'main,

                    Event::AppTerminating {
                        timestamp: _timestamp,
                    } => {
                        println!("App terminating...")
                    }

                    Event::AppLowMemory {
                        timestamp: _timestamp,
                    } => {
                        println!("App low memory!")
                    }

                    Event::AppWillEnterBackground {
                        timestamp: _timestamp,
                    } => {
                        println!("App will enter background...")
                    }

                    Event::AppDidEnterBackground {
                        timestamp: _timestamp,
                    } => {
                        println!("App did enter background...")
                    }

                    Event::AppWillEnterForeground {
                        timestamp: _timestamp,
                    } => {
                        println!("App will enter foreground...")
                    }

                    Event::AppDidEnterForeground {
                        timestamp: _timestamp,
                    } => {
                        println!("App did enter foreground...")
                    }

                    Event::Window {
                        timestamp: _timestamp,
                        window_id: _window_id,
                        win_event,
                    } => match &win_event {
                        WindowEvent::None => {
                            // println!("(Window {}) {:?}", window_id, &win_event);
                        }
                        WindowEvent::Shown | WindowEvent::Hidden => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        WindowEvent::Exposed => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        WindowEvent::Minimized | WindowEvent::Maximized => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        WindowEvent::Restored => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        WindowEvent::Moved(_, _) => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        /*WindowEvent::Resized(width, height)
                        | */
                        WindowEvent::SizeChanged(width, height) => {
                            let rendering_context = &self.context.rendering_context;

                            let mut canvas_window = rendering_context.canvas.borrow_mut();
                            let window_info = &mut (*self.window_info).borrow_mut();
                            let canvas_texture = &mut (*self.canvas_texture).borrow_mut();

                            let resolution = Resolution {
                                width: *width as u32,
                                height: *height as u32,
                            };

                            handle_window_resize_event(
                                &mut canvas_window,
                                window_info,
                                canvas_texture,
                                resolution,
                            )?;
                        }
                        // The cursor has entered or left the window boundary.
                        WindowEvent::Enter | WindowEvent::Leave => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        WindowEvent::FocusGained | WindowEvent::FocusLost => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        WindowEvent::Close => {
                            // println!("(Window {}) {:?}", window_id, &win_event)
                        }
                        _ => (),
                    },

                    Event::MouseMotion { xrel, yrel, .. } => {
                        mouse_state.relative_motion.0 = xrel;
                        mouse_state.relative_motion.1 = yrel;
                    }

                    Event::MouseWheel { direction, y, .. } => {
                        mouse_state.wheel_event.replace(MouseWheelEvent {
                            direction,
                            delta: y,
                        });
                    }

                    Event::KeyDown {
                        keycode: Some(keycode),
                        keymod: modifiers,
                        ..
                    } => match keycode {
                        Keycode::Escape => break 'main,
                        Keycode::F8 => self.toggle_updates(),
                        Keycode::F9 => should_update_step_forward = true,
                        Keycode::F11 => {
                            let windowing_mode = self.window_info.borrow().windowing_mode;

                            match windowing_mode {
                                AppWindowingMode::Windowed => {
                                    self.set_windowing_mode(AppWindowingMode::FullScreenWindowed)?;
                                }
                                AppWindowingMode::FullScreen
                                | AppWindowingMode::FullScreenWindowed => {
                                    self.set_windowing_mode(AppWindowingMode::Windowed)?;
                                }
                            }
                        }
                        _ => {
                            if keycode == Keycode::SPACE {
                                self.toggle_updates();
                            }

                            keyboard_state.keys_pressed.push((keycode, modifiers));
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

            let next_mouse_buttons_down: HashSet<MouseButton> =
                current_mouse_state.pressed_mouse_buttons().collect();

            mouse_state
                .prev_buttons_down
                .clone_from(&prev_mouse_buttons_down);

            mouse_state
                .buttons_down
                .clone_from(&next_mouse_buttons_down);

            // Get the difference between the old and new signals

            let old_mouse_clicks = &prev_mouse_buttons_down - &next_mouse_buttons_down;
            let new_mouse_clicks = &next_mouse_buttons_down - &prev_mouse_buttons_down;

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

            prev_mouse_buttons_down = next_mouse_buttons_down;

            prev_game_controller_state = game_controller.state;

            // Cache input device states

            mouse_state.prev_position = prev_mouse_position;
            mouse_state.prev_ndc_position = prev_mouse_ndc_position;

            mouse_state.position.0 = current_mouse_state.x();
            mouse_state.position.1 = current_mouse_state.y();

            {
                let window_info = self.window_info.borrow();

                mouse_state.ndc_position.0 =
                    mouse_state.position.0 as f32 / window_info.window_resolution.width as f32;

                mouse_state.ndc_position.1 =
                    mouse_state.position.1 as f32 / window_info.window_resolution.height as f32;
            }

            // Drag events.

            if mouse_state.buttons_down.contains(&MouseButton::Left)
                && mouse_state.prev_buttons_down.contains(&MouseButton::Left)
                && !(mouse_state.relative_motion.0 == 0 && mouse_state.relative_motion.1 == 0)
            {
                mouse_state.drag_event.replace(MouseDragEvent {
                    delta: mouse_state.relative_motion,
                });
            }

            // Update current scene

            if !self.are_updates_paused || should_update_step_forward {
                let ticks_since_last_update: u64 =
                    timer_subsystem.performance_counter() - last_update_tick;

                self.timing_info.seconds_since_last_update =
                    ticks_since_last_update as f32 / ticks_per_second as f32;

                self.timing_info.uptime_seconds += self.timing_info.seconds_since_last_update;
            }

            #[cfg(feature = "debug_cycle_counts")]
            self.cycle_counters
                .get_mut(AppCycleCounter::UpdateCallback as usize)
                .start();

            update(
                &mut self,
                &mut keyboard_state,
                &mut mouse_state,
                &mut game_controller.state,
            )?;

            #[cfg(feature = "debug_cycle_counts")]
            self.cycle_counters
                .get_mut(AppCycleCounter::UpdateCallback as usize)
                .end();

            prev_mouse_position = mouse_state.position;
            prev_mouse_ndc_position = mouse_state.ndc_position;

            last_update_tick = timer_subsystem.performance_counter();

            // Render current scene to the window canvas.

            {
                let mut canvas_window = self.context.rendering_context.canvas.borrow_mut();

                let mut canvas_texture = self.canvas_texture.borrow_mut();

                #[cfg(feature = "debug_cycle_counts")]
                let cycle_counters = Some(&mut self.cycle_counters);

                #[cfg(not(feature = "debug_cycle_counts"))]
                let cycle_counters = None;

                let current_frame_index = self.timing_info.current_frame_index;

                render_and_present(
                    &mut canvas_window,
                    &mut canvas_texture,
                    cycle_counters,
                    Some(current_frame_index),
                    None,
                    render,
                )?;
            }

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

            #[cfg(feature = "print_timing_info")]
            if frames_rendered % 50 == 0 {
                println!("FPS: {}", self.timing_info.frames_per_second);
            }

            frame_start = timer_subsystem.performance_counter();

            // Sleep if we can...

            // timer_subsystem.delay(self.timing_info.unused_milliseconds.floor() as u32);

            // @NOTE(mzalla) Will overflow, and that's okay.
            frames_rendered += 1;

            self.timing_info.current_frame_index = frames_rendered;

            #[cfg(feature = "debug_cycle_counts")]
            {
                self.cycle_counters
                    .get_mut(AppCycleCounter::Run as usize)
                    .end();

                #[cfg(feature = "print_timing_info")]
                println!("Frame {}:", self.timing_info.current_frame_index);

                self.cycle_counters.report::<AppCycleCounter>();
            }
        }

        Ok(())
    }
}

fn resize_window(
    canvas: &mut Canvas<Window>,
    window_info: &mut AppWindowInfo,
    new_resolution: Resolution,
) -> Result<(), String> {
    match canvas
        .window_mut()
        .set_size(new_resolution.width, new_resolution.height)
    {
        Ok(_) => {
            // Update window info.

            window_info.window_resolution = new_resolution;

            // println!("Resized application window to {}.", new_resolution);

            Ok(())
        }
        Err(e) => Err(format!("Failed to resize app window: {}", e)),
    }
}

fn resize_canvas(
    canvas: &mut Canvas<Window>,
    window_info: &mut AppWindowInfo,
    canvas_texture: &mut Texture,
    new_resolution: Resolution,
) -> Result<(), String> {
    // Re-allocates a window canvas for this window.

    let texture_creator = canvas.texture_creator();

    match make_canvas_texture(new_resolution, &texture_creator, None) {
        Ok(texture) => {
            *canvas_texture = texture;

            window_info.canvas_resolution = new_resolution;

            // println!("Resized canvas to {}.", new_resolution);

            Ok(())
        }
        Err(e) => Err(format!(
            "Failed to reallocate window canvas in App::resize_canvas(): {}",
            e
        )),
    }
}

pub fn handle_window_resize_event(
    canvas: &mut Canvas<Window>,
    window_info: &mut AppWindowInfo,
    canvas_texture: &mut Texture,
    new_resolution: Resolution,
) -> Result<(), String> {
    resize_window(canvas, window_info, new_resolution)?;
    resize_canvas(canvas, window_info, canvas_texture, new_resolution)
}

pub fn handle_set_window_mode_event(
    canvas: &mut Canvas<Window>,
    window_info: &mut AppWindowInfo,
    canvas_texture: &mut Texture,
    windowing_mode: AppWindowingMode,
) -> Result<(), String> {
    let fullscreen_type = match windowing_mode {
        AppWindowingMode::FullScreen => FullscreenType::True,
        AppWindowingMode::Windowed => FullscreenType::Off,
        AppWindowingMode::FullScreenWindowed => FullscreenType::Desktop,
    };

    let window = canvas.window_mut();

    window
        .set_fullscreen(fullscreen_type)
        .map_err(|e| format!("Failed to set the display mode: {}", e))?;

    // Update window info.

    window_info.windowing_mode = windowing_mode;

    // Resize our canvas.

    let resolution = Resolution::new(window.size());

    resize_canvas(canvas, window_info, canvas_texture, resolution)
}

fn render_and_present(
    canvas_window: &mut Canvas<Window>,
    canvas_texture: &mut Texture,
    mut cycle_counters: Option<&mut CycleCounters>,
    current_frame_index: Option<u32>,
    new_resolution: Option<Resolution>,
    render: &impl Fn(Option<u32>, Option<Resolution>, &mut [u8]) -> Result<(), String>,
) -> Result<(), String> {
    canvas_texture.with_lock(
        None,
        |write_only_byte_array, _pitch| -> Result<(), String> {
            // Render current scene

            if let Some(counters) = cycle_counters.as_mut() {
                counters
                    .get_mut(AppCycleCounter::RenderAndPresent as usize)
                    .start();
                counters
                    .get_mut(AppCycleCounter::RenderCallback as usize)
                    .start();
            }

            render(current_frame_index, new_resolution, write_only_byte_array)?;

            if let Some(counters) = cycle_counters.as_mut() {
                counters
                    .get_mut(AppCycleCounter::RenderCallback as usize)
                    .end();

                counters
                    .get_mut(AppCycleCounter::CopyAndPresent as usize)
                    .start();
            }

            Ok(())
        },
    )??;

    // Flip buffers

    // Note that Canvas<Window>::copy() will automatically stretch our
    // window canvas to fit the current window size, if `dst` is `None`.

    canvas_window.copy(canvas_texture, None, None)?;

    canvas_window.present();

    if let Some(counters) = cycle_counters.as_mut() {
        counters
            .get_mut(AppCycleCounter::CopyAndPresent as usize)
            .end();

        counters
            .get_mut(AppCycleCounter::RenderAndPresent as usize)
            .end();
    }

    Ok(())
}

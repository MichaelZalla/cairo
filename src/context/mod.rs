use std::sync::RwLock;

use sdl2::{
    render::{BlendMode, Canvas, Texture, TextureCreator},
    ttf::Sdl2TtfContext,
    video::{Window, WindowContext},
    EventPump, TimerSubsystem,
};

use crate::app::AppWindowInfo;

use super::device::GameController;

const GAME_CONTROLLER_COUNT: usize = 4;

pub struct ApplicationContext {
    pub rendering_context: ApplicationRenderingContext,
    pub ttf_context: &'static Sdl2TtfContext,
    pub screen_width: u32,
    pub screen_height: u32,
    pub timer: TimerSubsystem,
    pub game_controllers: Vec<Option<GameController>>,
    pub events: EventPump,
}

pub struct ApplicationRenderingContext {
    pub canvas: RwLock<Canvas<Window>>,
}

pub fn get_application_context(window_info: &AppWindowInfo) -> Result<ApplicationContext, String> {
    let sdl_context = sdl2::init()?;

    sdl_context.mouse().show_cursor(window_info.show_cursor);

    let timer = sdl_context.timer()?;

    let ttf_context: &'static Sdl2TtfContext;

    match sdl2::ttf::init() {
        Ok(context) => {
            println!("Initialized TTF font subsystem.");

            let boxed = Box::new(context);

            ttf_context = Box::leak(boxed);
        }
        Err(e) => {
            return Err(format!(
                "Error initializing TTF font subsystem: '{}'",
                e.to_string()
            ))
        }
    }

    let game_controller_subsystem = sdl_context.game_controller()?;

    let mut game_controllers: Vec<Option<GameController>> = vec![];

    for _ in 0..GAME_CONTROLLER_COUNT {
        game_controllers.push(None);
    }

    let count = game_controller_subsystem.num_joysticks()?;

    println!(
        "Initialized game controller subsystem with {} joysticks.",
        count
    );

    for joystick_index in 0..count {
        if game_controller_subsystem.is_game_controller(joystick_index) {
            match game_controller_subsystem.open(joystick_index) {
                Ok(joystick) => {
                    if joystick.attached() {
                        println!("Controller mapping: {}", joystick.mapping());

                        game_controllers[joystick_index as usize] =
                            Some(GameController::new_with_handle(joystick));
                    }
                }
                Err(e) => {
                    println!("Error initializing controller {}: '{}'", joystick_index, e)
                }
            }
        }
    }

    let haptic_subsystem = sdl_context.haptic()?;

    println!("Initialized haptic subsystem.");

    for controller in game_controllers.as_mut_slice() {
        if controller.is_some() {
            let unwrapped = controller.as_mut().unwrap();

            match haptic_subsystem.open_from_joystick_id(unwrapped.id) {
                Ok(device) => {
                    unwrapped.set_haptic_device(device);
                }
                Err(e) => {
                    println!(
                        "Error retrieving haptic device for joystick {}: '{}'",
                        unwrapped.id, e
                    );
                }
            }
        }
    }

    let events = sdl_context.event_pump()?;

    let video_subsystem = sdl_context.video()?;

    let mut window_builder = video_subsystem.window(
        &window_info.title,
        window_info.canvas_width,
        window_info.canvas_height,
    );

    // window_builder.opengl()
    // window_builder.position_centered()
    // window_builder.borderless();

    if window_info.full_screen {
        // Will verride `canvas_width` and `canvas_height` for the current
        // desktop resolution;
        window_builder.fullscreen_desktop();
    }

    match window_builder.build() {
        Ok(window) => {
            let screen_width = window.size().0;
            let screen_height = window.size().1;

            // Captures mouse movements even when mouse position is constrained
            // to the window border.
            sdl_context.mouse().set_relative_mouse_mode(true);

            // Begin with the cursor at the center of the viewport.
            sdl_context.mouse().warp_mouse_in_window(
                &window,
                (screen_width / 2) as i32,
                (screen_height / 2) as i32,
            );

            let rendering_context =
                get_application_rendering_context(window, window_info.vertical_sync).unwrap();

            Ok(ApplicationContext {
                screen_width,
                screen_height,
                rendering_context,
                ttf_context,
                timer,
                game_controllers,
                events,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_application_rendering_context<'a, 'r>(
    window: Window,
    vertical_sync: bool,
) -> Result<ApplicationRenderingContext, String> {
    if vertical_sync {
        match window.into_canvas().present_vsync().build() {
            Ok(canvas) => {
                return Ok(ApplicationRenderingContext {
                    canvas: RwLock::new(canvas),
                });
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        match window.into_canvas().build() {
            Ok(canvas) => {
                return Ok(ApplicationRenderingContext {
                    canvas: RwLock::new(canvas),
                });
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn get_backbuffer<'r>(
    canvas_width: u32,
    canvas_height: u32,
    texture_creator: &'r TextureCreator<WindowContext>,
    blend_mode: BlendMode,
) -> Result<Texture, String> {
    match texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::RGBA32,
        canvas_width,
        canvas_height,
    ) {
        Ok(mut backbuffer) => {
            const BYTES_PER_PIXEL: u32 = 4;

            let canvas_pitch: u32 = canvas_width * BYTES_PER_PIXEL;

            let pixel_buffer_size: usize =
                (canvas_width * canvas_height * BYTES_PER_PIXEL) as usize;
            let pixel_buffer = &vec![0; pixel_buffer_size];

            match backbuffer.update(None, pixel_buffer, canvas_pitch as usize) {
                Ok(_) => {
                    backbuffer.set_blend_mode(blend_mode);

                    return Ok(backbuffer);
                }
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

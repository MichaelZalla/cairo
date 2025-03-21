use std::{cell::RefCell, rc::Rc};

use sdl2::{
    image::InitFlag,
    render::{BlendMode, Canvas, Texture, TextureCreator},
    ttf::Sdl2TtfContext,
    video::{Window, WindowContext},
    EventSubsystem, Sdl,
};

use crate::{
    app::window::AppWindowingMode, app::AppWindowInfo, device::game_controller::GameController,
};

use super::resolution::Resolution;

const GAME_CONTROLLER_COUNT: usize = 4;

pub struct ApplicationContext {
    pub sdl_context: Sdl,
    pub rendering_context: ApplicationRenderingContext,
    pub ttf_context: &'static Sdl2TtfContext,
    pub event_subsystem: EventSubsystem,
    pub screen_width: u32,
    pub screen_height: u32,
    pub game_controllers: Vec<Option<GameController>>,
}

pub struct ApplicationRenderingContext {
    pub canvas: Rc<RefCell<Canvas<Window>>>,
}

pub fn make_application_context(window_info: &AppWindowInfo) -> Result<ApplicationContext, String> {
    let sdl_context = sdl2::init()?;

    println!("Initialized SDL v{}.", sdl2::version::version());

    match sdl2::image::init(InitFlag::JPG | InitFlag::PNG) {
        Ok(_) => {
            println!(
                "Initialized SDL_Image v{}.",
                sdl2::image::get_linked_version()
            );
        }
        Err(_) => return Err("Failed to initialize SDL_Image library from DLL.".to_string()),
    }

    let event_subsystem = sdl_context.event().unwrap();

    sdl_context.mouse().show_cursor(window_info.show_cursor);

    let ttf_context: &'static Sdl2TtfContext = match sdl2::ttf::init() {
        Ok(context) => {
            println!("Initialized SDL2_ttf v{}.", sdl2::ttf::get_linked_version());

            let boxed = Box::new(context);

            Box::leak(boxed)
        }
        Err(e) => {
            return Err(format!(
                "Failed to initialize SDL2_ttf library from DLL: {}",
                e
            ))
        }
    };

    let game_controller_subsystem = sdl_context.game_controller()?;

    let mut game_controllers: Vec<Option<GameController>> = vec![];

    for _ in 0..GAME_CONTROLLER_COUNT {
        game_controllers.push(None);
    }

    let count = game_controller_subsystem.num_joysticks()?;

    #[cfg(feature = "print_init_info")]
    println!(
        "Initialized game controller subsystem with {} joysticks.\n",
        count
    );

    for joystick_index in 0..count {
        if game_controller_subsystem.is_game_controller(joystick_index) {
            match game_controller_subsystem.open(joystick_index) {
                Ok(joystick) => {
                    if joystick.attached() {
                        #[cfg(feature = "print_init_info")]
                        println!("Controller mapping: {}", joystick.mapping());

                        game_controllers[joystick_index as usize] =
                            Some(GameController::new_with_handle(joystick));
                    }
                }
                #[allow(unused)]
                Err(e) => {
                    #[cfg(feature = "print_init_info")]
                    println!("Error initializing controller {}: '{}'", joystick_index, e)
                }
            }
        }
    }

    let haptic_subsystem = sdl_context.haptic()?;

    #[cfg(feature = "print_init_info")]
    println!("Initialized haptic subsystem.\n");

    for controller in game_controllers.as_mut_slice() {
        if controller.is_some() {
            let unwrapped = controller.as_mut().unwrap();

            match haptic_subsystem.open_from_joystick_id(unwrapped.id) {
                Ok(device) => {
                    unwrapped.set_haptic_device(device);
                }
                #[allow(unused)]
                Err(e) => {
                    #[cfg(feature = "print_init_info")]
                    println!(
                        "Error retrieving haptic device for joystick {}: '{}'",
                        unwrapped.id, e
                    );
                }
            }
        }
    }

    let video_subsystem = sdl_context.video()?;

    let mut window_builder = video_subsystem.window(
        &window_info.title,
        window_info.window_resolution.width,
        window_info.window_resolution.height,
    );

    window_builder.position_centered();

    if window_info.resizable {
        window_builder.resizable();
    }

    match window_info.windowing_mode {
        AppWindowingMode::Windowed => {
            // Do nothing.
        }
        AppWindowingMode::FullScreen => {
            // Will override `canvas_resolution.width` and
            // `canvas_resolution.height` for the current desktop resolution;

            window_builder.fullscreen();
        }
        AppWindowingMode::FullScreenWindowed => {
            // Will override `canvas_resolution.width` and
            // `canvas_resolution.height` for the current desktop resolution;

            window_builder.fullscreen_desktop();
        }
    }

    match window_builder.build() {
        Ok(window) => {
            let screen_width = window.size().0;
            let screen_height = window.size().1;

            // Captures mouse movements even when mouse position is constrained
            // to the window border.
            sdl_context
                .mouse()
                .set_relative_mouse_mode(window_info.relative_mouse_mode);

            // Begin with the cursor at the center of the viewport.
            sdl_context.mouse().warp_mouse_in_window(
                &window,
                (screen_width / 2) as i32,
                (screen_height / 2) as i32,
            );

            let rendering_context =
                get_application_rendering_context(window, window_info.vertical_sync).unwrap();

            Ok(ApplicationContext {
                sdl_context,
                event_subsystem,
                screen_width,
                screen_height,
                rendering_context,
                ttf_context,
                game_controllers,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_application_rendering_context(
    window: Window,
    vertical_sync: bool,
) -> Result<ApplicationRenderingContext, String> {
    let mut canvas_builder = window.into_canvas();

    if vertical_sync {
        canvas_builder = canvas_builder.present_vsync();
    }

    match canvas_builder.build() {
        Ok(canvas) => Ok(ApplicationRenderingContext {
            canvas: Rc::new(RefCell::new(canvas)),
        }),
        Err(e) => Err(e.to_string()),
    }
}

pub fn make_canvas_texture(
    canvas_resolution: Resolution,
    texture_creator: &TextureCreator<WindowContext>,
    blend_mode: Option<BlendMode>,
) -> Result<Texture, String> {
    match texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::RGBA32,
        canvas_resolution.width,
        canvas_resolution.height,
    ) {
        Ok(mut canvas_texture) => {
            const BYTES_PER_PIXEL: u32 = 4;

            let canvas_pitch: u32 = canvas_resolution.width * BYTES_PER_PIXEL;

            let pixel_buffer_size: usize =
                (canvas_resolution.width * canvas_resolution.height * BYTES_PER_PIXEL) as usize;
            let pixel_buffer = &vec![0; pixel_buffer_size];

            match canvas_texture.update(None, pixel_buffer, canvas_pitch as usize) {
                Ok(_) => {
                    let mode = match blend_mode {
                        Some(mode) => mode,
                        None => BlendMode::None,
                    };

                    canvas_texture.set_blend_mode(mode);

                    Ok(canvas_texture)
                }
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

use sdl2::{
	video::{Window, WindowContext},
	TimerSubsystem,
	EventPump,
	render::{Canvas, TextureCreator, BlendMode, Texture},
};

use super::device::GameController;

pub struct ApplicationContext {
	pub window: Window,
	pub timer: TimerSubsystem,
	pub game_controllers: Vec<Option<GameController>>,
	pub events: EventPump,
}

pub struct ApplicationRenderingContext {
	pub canvas: Canvas<Window>,
}

pub fn get_application_context(
	window_title: &str,
	window_width: u32,
	window_height: u32,
	full_screen: bool,
	show_cursor: bool,
) -> Result<ApplicationContext, String>
{

	match sdl2::init() {
		Ok(sdl_context) => {

			sdl_context
				.mouse()
				.show_cursor(show_cursor);

			match sdl_context.timer() {
				Ok(timer) => {

					match sdl_context.game_controller() {
						Ok(game_controller_subsystem) => {

							let mut game_controllers: Vec<Option<GameController>> = vec![];

							for _ in 0..4 {
								game_controllers.push(None);
							}

							match game_controller_subsystem.num_joysticks() {
								Ok(count) => {

									println!("Initialized game controller subsystem with {} joysticks.", count);

									for joystick_index in 0..count {

										if game_controller_subsystem.is_game_controller(joystick_index) {

											match game_controller_subsystem.open(joystick_index) {
												Ok(joystick) => {

													if joystick.attached() {

														println!("Controller mapping: {}", joystick.mapping());

														game_controllers[joystick_index as usize] = Some(
															GameController::new_with_handle(joystick)
														);
													}
												},
												Err(e) => {
													println!("Error initializing controller {}: '{}'", joystick_index, e)
												}
											}

										}

									}

								},
								Err(e) => {
									println!("Error initializing game controller subsystem: '{}'", e);
								},
							}

							match sdl_context.event_pump() {
								Ok(events) => {

									match sdl_context.video() {
										Ok(video_subsystem) => {

											let mut window_builder = video_subsystem.window(
												window_title,
												window_width,
												window_height
											);

											// window_builder.opengl()
											// window_builder.position_centered()
											// window_builder.borderless();

											if full_screen {
												// @NOTE(mzalla) Overrides
												// `window_width` and `window_height`
												// for the current desktop resolution;
												window_builder.fullscreen_desktop();
											}

											match window_builder.build() {
												Ok(window) => Ok(ApplicationContext{
													window: window,
													timer: timer,
													game_controllers,
													events: events,
												}),
												Err(e) => Err(e.to_string()),
											}

										},
										Err(e) => Err(e),
									}

								},
								Err(e) => Err(e),
							}

						},
						Err(e) => Err(e),
					}

				},
				Err(e) => Err(e),
			}

		},
		Err(e) => Err(e),
	}

}

pub fn get_application_rendering_context<'a,'r>(
	window: Window) -> Result<ApplicationRenderingContext, String>
{
	match window
		.into_canvas()
		// .accelerated()
		// .present_vsync()
		.build()
	{
		Ok(canvas) => {
			return Ok(ApplicationRenderingContext{
				canvas: canvas,
			});
		},
		Err(e) => Err(e.to_string()),
	}
}

pub fn get_backbuffer<'r>(
	context: &ApplicationRenderingContext,
	texture_creator: &'r TextureCreator<WindowContext>,
	blend_mode: BlendMode) -> Result<Texture<'r>, String>
{

	let size = context.canvas.output_size().unwrap();

	let canvas_width = size.0;
	let canvas_height = size.1;

	match texture_creator
		.create_texture_streaming(
			sdl2::pixels::PixelFormatEnum::RGBA32,
			canvas_width,
			canvas_height)
	{
		Ok(mut backbuffer) => {

			const BYTES_PER_PIXEL: u32 = 4;

			let canvas_pitch: u32 = canvas_width * BYTES_PER_PIXEL;

			let pixel_buffer_size: usize = (canvas_width * canvas_height * BYTES_PER_PIXEL) as usize;
			let pixel_buffer = &vec![0; pixel_buffer_size];

			match backbuffer.update(
				None,
				pixel_buffer,
				canvas_pitch as usize)
			{
				Ok(_) => {

					backbuffer.set_blend_mode(blend_mode);

					return Ok(backbuffer);

				},
				Err(e) => Err(e.to_string()),
			}

		},
		Err(e) => Err(e.to_string())
	}

}

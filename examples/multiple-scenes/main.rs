extern crate sdl2;

use std::{
	sync::RwLock,
	cmp::min,
	cell::RefCell,
};

use cairo::{
    app::App,
    fs::get_absolute_filepath,
    device::{
        MouseState,
        KeyboardState,
        GameControllerState,
    },
    graphics::{Graphics, PixelBuffer},
	mesh::get_mesh_from_obj,
	entity::Entity, scenes::default_scene::DefaultScene,
	scene::Scene,
};
use sdl2::keyboard::Keycode;

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {

	let graphics = Graphics{
		buffer: PixelBuffer{
			width: WINDOW_WIDTH,
			height: WINDOW_HEIGHT,
			width_over_height: ASPECT_RATIO,
			height_over_width: 1.0 / ASPECT_RATIO,
			pixels: vec![0 as u32; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize],
		},
	};
	
	let cube_mesh = get_mesh_from_obj(get_absolute_filepath("/data/obj/cube.obj"));
	let mut cube_entity = Entity::new(&cube_mesh);

	let teapot_mesh = get_mesh_from_obj(get_absolute_filepath("/data/obj/teapot.obj"));
	let mut teapot_entity = Entity::new(&teapot_mesh);

	let entities: Vec<&mut Entity> = vec![
		&mut cube_entity,
	];

	let entities2 = vec![
		&mut teapot_entity,
	];

	let entities_rwl = RwLock::new(entities);
	let entities2_rwl = RwLock::new(entities2);

	let scenes = RefCell::new(
		vec![
			DefaultScene::new(
				graphics.clone(),
				&entities_rwl,
			),
			DefaultScene::new(
				graphics.clone(),
				&entities2_rwl
			),
		]
	);

	let current_scene_index = RefCell::new(
		min(0, scenes.borrow().len() - 1)
	);

	// Set up our app

	let update = |
		keyboard_state: &KeyboardState,
		mouse_state: &MouseState,
		game_controller_state: &GameControllerState,
		delta_t_seconds: f32| -> ()
	{                
		// Update scene
				
		let scenes_len = scenes.borrow_mut().len();
		
		let mut new_index = *current_scene_index.borrow();

		for keycode in keyboard_state.keys_pressed.to_owned() {
			match keycode {
				Keycode::Num4 { .. } => {
					new_index = min(scenes_len - 1, 0);
				},
				Keycode::Num5 { .. } => {
					new_index = min(scenes_len - 1, 1);
				},
				_ => {

				}
			}
		}

		*current_scene_index.borrow_mut() = new_index;

		scenes.borrow_mut()[*current_scene_index.borrow()].update(
			&keyboard_state,
			&mouse_state,
			&game_controller_state,
			delta_t_seconds
		);
	};

	let render = || -> Result<Vec<u32>, String>
	{
		// Render current scene

		scenes.borrow_mut()[*current_scene_index.borrow()].render();

		// @TODO(mzalla) Return reference to a captured variable???
		return Ok(
			scenes.borrow_mut()[*current_scene_index.borrow()].get_pixel_data().clone()
		);
	};

	let app = App::new(
		"examples/multiple-scenes",
		WINDOW_WIDTH,
		ASPECT_RATIO,
		update, 
		render,
	);

	app.run()?;

    Ok(())

}

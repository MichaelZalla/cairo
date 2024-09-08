use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    color::{self, Color},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
};

use graph::{Graph, GraphingFunction};

mod graph;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/numerical-methods".to_string(),
        resizable: true,
        ..Default::default()
    };

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        Some(color::BLACK.to_u32()),
    );

    let graph = Graph::new(
        (
            (framebuffer.width / 2) as i32,
            (framebuffer.height / 2) as i32,
        ),
        48,
    );

    let graph_rc = RefCell::new(graph);

    let framebuffer_rc = RefCell::new(framebuffer);

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        let graph = graph_rc.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        if let Some(resolution) = new_resolution {
            framebuffer.resize(resolution.width, resolution.height);
        }

        framebuffer.clear(None);

        let functions: Vec<(GraphingFunction, Color)> = vec![
            (|x: f32| -> f32 { x.sin() }, color::BLUE),
            (|x: f32| -> f32 { x.cos() }, color::RED),
            (|x: f32| -> f32 { x * x }, color::GREEN),
            (|x: f32| -> f32 { x.sqrt() }, color::SKY_BOX),
            (|x: f32| -> f32 { x.exp() }, color::ORANGE),
        ];

        graph.render(&functions, &mut framebuffer);

        Ok(framebuffer.get_all().clone())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let mut update = |_app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let mut graph = graph_rc.borrow_mut();

        graph.update(keyboard_state, mouse_state);

        Ok(())
    };

    let render = |frame_index, new_resolution| -> Result<Vec<u32>, String> {
        render_scene_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}

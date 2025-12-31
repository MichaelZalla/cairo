extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{
        App, AppWindowInfo,
        resolution::{RESOLUTION_1280_BY_720, Resolution},
    },
    buffer::Buffer2D,
    color::{self},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
};

use crate::{
    graph::Graph,
    simulation::{PIDControllerFlags, Simulation},
};

mod graph;
mod simulation;

#[derive(Default)]
struct DemoState {
    graph: RefCell<Graph>,
    simulations: Vec<RefCell<Simulation>>,
    framebuffer: RefCell<Buffer2D>,
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/pid-controller".to_string(),
        window_resolution: RESOLUTION_1280_BY_720,
        canvas_resolution: RESOLUTION_1280_BY_720,
        relative_mouse_mode: false,
        ..Default::default()
    };

    // Set up our app

    let framebuffer: Buffer2D<u32> = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let center = window_info.canvas_resolution.center();

    let graph = Graph::new((center.0 as i32, center.1 as i32), 100);

    let state = DemoState {
        framebuffer: RefCell::new(framebuffer),
        graph: RefCell::new(graph),
        simulations: vec![
            // RefCell::new(Simulation::new(PIDControllerFlags::P, color::RED)),
            // RefCell::new(Simulation::new(
            //     PIDControllerFlags::P | PIDControllerFlags::I,
            //     color::GREEN,
            // )),
            RefCell::new(Simulation::new(
                PIDControllerFlags::P | PIDControllerFlags::I | PIDControllerFlags::D,
                color::BLUE,
            )),
        ],
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let mut framebuffer = state.framebuffer.borrow_mut();

        let graph = state.graph.borrow_mut();

        // Clears pixel buffer

        if let Some(resolution) = new_resolution {
            framebuffer.resize(resolution.width, resolution.height);
        }

        framebuffer.clear(None);

        graph.axes(&mut framebuffer);

        // For each simulation

        for simulation_rc in &state.simulations {
            let simulation = simulation_rc.borrow_mut();

            simulation.render(&graph, &mut framebuffer);
        }

        // Blit our framebuffer to the native window's surface.

        framebuffer.copy_to(canvas);

        Ok(())
    };

    // Create and run our app.

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      _mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let delta_t = app.timing_info.seconds_since_last_update;

        if delta_t > 0.0 {
            for simulation_rc in state.simulations.iter() {
                simulation_rc.borrow_mut().tick(delta_t);
            }
        }

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}

use std::{cell::RefCell, f32::consts::TAU};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
};

use graph::{BoxedGraphingFunction, Graph};
use integrator::{integrate_forward_euler, integrate_rk2, integrate_rk4};
use state::{State, StateDerivative};

mod graph;
mod integrator;
mod state;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/numerical-methods".to_string(),
        relative_mouse_mode: false,
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

    // Exponential decay.

    static X_0: f32 = 4.0;
    static TIME_CONSTANT_OF_DECAY: f32 = 2.5; // Initial slope = (-x0 / TIME_CONSTANT_OF_DECAY).

    let exp_decay: BoxedGraphingFunction =
        Box::new(|t: f32| -> f32 { X_0 * (-t / TIME_CONSTANT_OF_DECAY).exp() });

    let exp_decay_system_dynamics_function =
        Box::leak(Box::new(|state: State| -> StateDerivative {
            StateDerivative {
                // v(t) = (-1 / T) * x
                f0: (-1.0 / TIME_CONSTANT_OF_DECAY) * state.f0,
                f1: 0.0,
            }
        }));

    // Sinusoidal oscillation.

    static MAGNITUDE: f32 = 2.0;
    static PERIOD: f32 = 0.5 * TAU;
    static PHASE_ANGLE: f32 = 0.0;
    static FREQUENCY: f32 = TAU / PERIOD;

    let oscillation: BoxedGraphingFunction =
        Box::new(|t: f32| -> f32 { MAGNITUDE * (TAU * (t / PERIOD) - PHASE_ANGLE).cos() });

    let oscillation_system_dynamics_function =
        Box::leak(Box::new(|state: State| -> StateDerivative {
            // E_k = (1/2) * M * V^2
            // a(t) = -(F^2) * x - D * v
            static DAMPENING: f32 = 0.005;

            StateDerivative {
                f0: state.f1,                                                   // Velocity
                f1: -(FREQUENCY * FREQUENCY) * state.f0 - DAMPENING * state.f1, // Acceleration
            }
        }));

    // Define app callbacks.

    let framebuffer_rc = RefCell::new(framebuffer);

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let graph = graph_rc.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        if let Some(resolution) = new_resolution {
            framebuffer.resize(resolution.width, resolution.height);
        }

        framebuffer.clear(None);

        graph.axes(&mut framebuffer);

        // let functions: Vec<(BoxedGraphingFunction, Color)> = vec![
        //     (Box::new(|x: f32| -> f32 { x.sin() * 4.0 }), color::BLUE),
        //     (Box::new(|x: f32| -> f32 { x.cos() }), color::RED),
        //     (Box::new(|x: f32| -> f32 { x * x }), color::GREEN),
        //     (Box::new(|x: f32| -> f32 { x.sqrt() }), color::SKY_BOX),
        //     (Box::new(|x: f32| -> f32 { x.exp() }), color::ORANGE),
        // ];

        // graph.functions(&functions, &mut framebuffer);
        // graph.point(1.0, 1.0, color::ORANGE, &mut framebuffer);
        // graph.line(0.0, 0.0, 5.0, 3.0, color::ORANGE, &mut framebuffer);

        if false {
            // Graph the exact solution for exponential decay.

            graph.function(&exp_decay, color::WHITE, &mut framebuffer);

            graph.line(
                0.0,
                X_0,
                TIME_CONSTANT_OF_DECAY,
                0.0,
                color::DARK_GRAY,
                &mut framebuffer,
            );

            // Graph approximate solutions using our different integrators.

            // let step_size = 2.0 * TIME_CONSTANT_OF_DECAY;
            let step_size = TIME_CONSTANT_OF_DECAY;
            // let step_size = TIME_CONSTANT_OF_DECAY / 2.0;
            // let step_size = TIME_CONSTANT_OF_DECAY / 4.0;

            let state_0 = State { f0: X_0, f1: 0.0 };

            // Approximate the solution using basic Euler (O(h^2)).

            // h <= 2*T
            // 2*T = 5.0

            integrate_forward_euler(
                state_0,
                exp_decay_system_dynamics_function,
                step_size,
                (10.0 / step_size) as usize,
                &graph,
                color::YELLOW,
                &mut framebuffer,
            );

            // Approximate the solution using RK2 (O(h^3)).

            // h <= 2*T
            // 2*T = 5.0

            integrate_rk2(
                state_0,
                exp_decay_system_dynamics_function,
                step_size,
                (10.0 / step_size) as usize,
                &graph,
                color::GREEN,
                &mut framebuffer,
            );

            // Approximate the solution using RK4 (O(h^5)).

            // h <= 2.78*T
            // 2*T = 6.95

            integrate_rk4(
                state_0,
                exp_decay_system_dynamics_function,
                step_size,
                (10.0 / step_size) as usize,
                &graph,
                color::BLUE,
                &mut framebuffer,
            );
        }

        if true {
            // Graph the exact solution for sinusoidal oscillation.

            graph.function(&oscillation, color::WHITE, &mut framebuffer);

            // Graph approximate solutions using our different integrators.

            // let step_size = 2.0 * PERIOD;
            // let step_size = PERIOD;
            // let step_size = PERIOD / 2.0;
            // let step_size = PERIOD / 4.0;
            // let step_size = PERIOD / 8.0;
            let step_size = PERIOD / 16.0;
            // let step_size = PERIOD / 128.0;

            let state_0 = State {
                f0: MAGNITUDE,
                f1: 0.0,
            };

            // Approximate the solution using basic Euler (O(h^2)).

            integrate_forward_euler(
                state_0,
                oscillation_system_dynamics_function,
                step_size,
                (20.0 / step_size) as usize,
                &graph,
                color::YELLOW,
                &mut framebuffer,
            );

            integrate_rk2(
                state_0,
                oscillation_system_dynamics_function,
                step_size,
                (20.0 / step_size) as usize,
                &graph,
                color::GREEN,
                &mut framebuffer,
            );

            integrate_rk4(
                state_0,
                oscillation_system_dynamics_function,
                step_size,
                (20.0 / step_size) as usize,
                &graph,
                color::BLUE,
                &mut framebuffer,
            );
        }

        framebuffer.copy_to(canvas);

        Ok(())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let mut update = |_app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let mut graph = graph_rc.borrow_mut();

        graph.update(keyboard_state, mouse_state);

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}

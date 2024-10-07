use cairo::{buffer::Buffer2D, color::Color};

use crate::{
    graph::Graph,
    state::{State, StateDerivative},
};

fn forward_euler(
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    state: State,
    h: f32,
) -> StateDerivative {
    state + system_dynamics_function(state) * h
}

fn rk2(
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    state: State,
    h: f32,
) -> StateDerivative {
    let k1 = system_dynamics_function(state);
    let k2 = system_dynamics_function(state + k1 * (h / 2.0));

    state + k2 * h
}

fn rk4(
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    state: State,
    h: f32,
) -> StateDerivative {
    let k1 = system_dynamics_function(state);
    let k2 = system_dynamics_function(state + k1 * (h / 2.0));
    let k3 = system_dynamics_function(state + k2 * (h / 2.0));
    let k4 = system_dynamics_function(state + k3 * h);

    state + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (h / 6.0)
}

#[allow(clippy::too_many_arguments)]
fn integrate_with_method(
    s_0: State,
    integrator: impl Fn(&dyn Fn(State) -> StateDerivative, State, f32) -> StateDerivative,
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: &Color,
    buffer: &mut Buffer2D,
) {
    let h = step_size;
    let mut t = 0.0;
    let mut current_state = s_0;

    for _ in 0..steps {
        let new_state = integrator(system_dynamics_function, current_state, h);

        graph.line(t, current_state.f0, t + h, new_state.f0, color, buffer);

        t += h;

        current_state = new_state;
    }
}

pub(crate) fn integrate_forward_euler(
    s_0: State,
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: &Color,
    buffer: &mut Buffer2D,
) {
    integrate_with_method(
        s_0,
        forward_euler,
        system_dynamics_function,
        step_size,
        steps,
        graph,
        color,
        buffer,
    )
}

pub(crate) fn integrate_rk2(
    s_0: State,
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: &Color,
    buffer: &mut Buffer2D,
) {
    integrate_with_method(
        s_0,
        rk2,
        system_dynamics_function,
        step_size,
        steps,
        graph,
        color,
        buffer,
    )
}

pub(crate) fn integrate_rk4(
    s_0: State,
    system_dynamics_function: &dyn Fn(State) -> StateDerivative,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: &Color,
    buffer: &mut Buffer2D,
) {
    integrate_with_method(
        s_0,
        rk4,
        system_dynamics_function,
        step_size,
        steps,
        graph,
        color,
        buffer,
    )
}

use cairo::{
    buffer::Buffer2D,
    color::Color,
    physics::simulation::integration::{
        forward_euler, rk2, rk4, IntegrationMethod, SystemDynamicsFunction,
    },
};

use crate::{graph::Graph, state::State};

#[allow(clippy::too_many_arguments)]
fn integrate_with_method(
    s_0: State,
    integrator: IntegrationMethod<State>,
    system_dynamics_function: SystemDynamicsFunction<State>,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: Color,
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
    system_dynamics_function: SystemDynamicsFunction<State>,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: Color,
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
    system_dynamics_function: SystemDynamicsFunction<State>,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: Color,
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
    system_dynamics_function: SystemDynamicsFunction<State>,
    step_size: f32,
    steps: usize,
    graph: &Graph,
    color: Color,
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

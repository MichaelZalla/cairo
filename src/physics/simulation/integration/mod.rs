use std::ops::{Add, Mul};

pub fn forward_euler<S>(system_dynamics_function: &dyn Fn(S) -> S, state: S, h: f32) -> S
where
    S: Copy + Add<Output = S> + Mul<Output = S> + Mul<f32, Output = S>,
{
    state + system_dynamics_function(state) * h
}

pub fn rk2<S>(system_dynamics_function: &dyn Fn(S) -> S, state: S, h: f32) -> S
where
    S: Copy + Add<Output = S> + Mul<Output = S> + Mul<f32, Output = S>,
{
    let k1 = system_dynamics_function(state);
    let k2 = system_dynamics_function(state + k1 * (h * 0.5));

    state + k2 * h
}

pub fn rk4<S>(system_dynamics_function: &dyn Fn(S) -> S, state: S, h: f32) -> S
where
    S: Copy + Add<Output = S> + Mul<Output = S> + Mul<f32, Output = S>,
{
    let k1 = system_dynamics_function(state);
    let k2 = system_dynamics_function(state + k1 * (h * 0.5));
    let k3 = system_dynamics_function(state + k2 * (h * 0.5));
    let k4 = system_dynamics_function(state + k3 * h);

    state + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (h * 0.166_666_67)
}

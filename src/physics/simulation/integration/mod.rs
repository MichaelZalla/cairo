use std::ops::{Add, Mul};

pub type SystemDynamicsFunction<S> = fn(S) -> S;

pub type IntegrationMethod<S> = fn(sdf: SystemDynamicsFunction<S>, state: S, h: f32) -> S;

pub fn forward_euler<S>(sdf: SystemDynamicsFunction<S>, state: S, h: f32) -> S
where
    S: Copy + Add<Output = S> + Mul<Output = S> + Mul<f32, Output = S>,
{
    state + sdf(state) * h
}

pub fn rk2<S>(sdf: SystemDynamicsFunction<S>, state: S, h: f32) -> S
where
    S: Copy + Add<Output = S> + Mul<Output = S> + Mul<f32, Output = S>,
{
    let k1 = sdf(state);
    let k2 = sdf(state + k1 * (h * 0.5));

    state + k2 * h
}

pub fn rk4<S>(sdf: SystemDynamicsFunction<S>, state: S, h: f32) -> S
where
    S: Copy + Add<Output = S> + Mul<Output = S> + Mul<f32, Output = S>,
{
    let k1 = sdf(state);
    let k2 = sdf(state + k1 * (h * 0.5));
    let k3 = sdf(state + k2 * (h * 0.5));
    let k4 = sdf(state + k3 * h);

    state + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * h * (1.0 / 6.0)
}

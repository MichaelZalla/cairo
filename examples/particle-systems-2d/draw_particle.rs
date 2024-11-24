use cairo::{
    animation::lerp,
    buffer::Buffer2D,
    color::{self, Color},
    graphics::Graphics,
    physics::simulation::particle::Particle,
    vec::vec3::Vec3,
};

use crate::particle::{MAX_PARTICLE_SIZE_PIXELS, PARTICLE_MAX_AGE_SECONDS};

pub(crate) fn draw_particle(
    particle: &Particle,
    screen_space_position: &Vec3,
    framebuffer: &mut Buffer2D,
) {
    debug_assert!(particle.alive);

    let age_alpha = particle.age / PARTICLE_MAX_AGE_SECONDS;

    let color = Color::from_vec3(lerp(
        color::WHITE.to_vec3(),
        color::BLACK.to_vec3(),
        age_alpha,
    ));

    // Assumes screen space position lies within our buffer.

    let x = screen_space_position.x as i32;
    let y = screen_space_position.y as i32;

    let size = MAX_PARTICLE_SIZE_PIXELS as f32 - ((MAX_PARTICLE_SIZE_PIXELS as f32) * age_alpha);
    let size_over_2 = size / 2.0;

    if let Some((x, y, width, height)) = Graphics::clip_rectangle(
        framebuffer,
        x - size_over_2 as i32,
        y - size_over_2 as i32,
        size as u32,
        size as u32,
    ) {
        Graphics::rectangle(framebuffer, x, y, width, height, Some(color.to_u32()), None)
    }
}

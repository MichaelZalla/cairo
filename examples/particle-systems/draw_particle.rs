use cairo::{
    animation::lerp,
    buffer::Buffer2D,
    color::{self, Color},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use crate::particle::{Particle, MAX_PARTICLE_SIZE_PIXELS, PARTICLE_MAX_AGE_SECONDS};

static PIXELS_PER_METER: f32 = 4.0;

pub(crate) fn screen_to_world_space(
    screen_space_position: &Vec3,
    framebuffer_center: &Vec3,
) -> Vec3 {
    let mut world_space_position = *screen_space_position;

    world_space_position = (world_space_position - *framebuffer_center) / PIXELS_PER_METER;

    world_space_position.y *= -1.0;

    world_space_position
}

pub(crate) fn world_to_screen_space(
    world_space_position: &Vec3,
    framebuffer_center: &Vec3,
) -> Vec3 {
    let mut screen_space_position = *world_space_position;

    screen_space_position.y *= -1.0;

    screen_space_position * PIXELS_PER_METER + *framebuffer_center
}

pub(crate) fn draw_particle(
    particle: &Particle,
    screen_space_position: &Vec3,
    _prev_screen_space_position: &Vec3,
    framebuffer: &mut Buffer2D,
) {
    debug_assert!(particle.alive);

    let age_alpha = particle.age / PARTICLE_MAX_AGE_SECONDS;

    let color = Color::from_vec3(lerp(
        color::WHITE.to_vec3(),
        color::BLACK.to_vec3(),
        age_alpha,
    ));

    // Color {
    //     r: age_alpha_u8,
    //     g: age_alpha_u8,
    //     b: age_alpha_u8,
    //     a: 1.0,
    // };

    // framebuffer.set(
    //     screen_space_position.x as u32,
    //     screen_space_position.y as u32,
    //     color.to_u32(),
    // );

    // Graphics::line(
    //     framebuffer,
    //     screen_space_position.x as i32,
    //     screen_space_position.y as i32,
    //     prev_screen_space_position.x as i32,
    //     prev_screen_space_position.y as i32,
    //     &color,
    // );

    // Assumes screen space position lies within our buffer.

    let x = screen_space_position.x as i32;
    let y = screen_space_position.y as i32;

    let size = MAX_PARTICLE_SIZE_PIXELS as f32 - ((MAX_PARTICLE_SIZE_PIXELS as f32) * age_alpha);
    let size_over_2 = size / 2.0;

    if let Some((x, y, width, height)) = Graphics::clip_rectangle(
        x - size_over_2 as i32,
        y - size_over_2 as i32,
        size as u32,
        size as u32,
        &framebuffer,
    ) {
        Graphics::rectangle(framebuffer, x, y, width, height, Some(&color), None)
    }
}

use cairo::{
    animation::lerp,
    buffer::Buffer2D,
    color::{self, Color},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use crate::particle::{Particle, PARTICLE_MAX_AGE_SECONDS};

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

    // let age_alpha_u8 = 255.0 * (1.0 - age_alpha.min(1.0));

    let color = Color::from_vec3(lerp(color::RED.to_vec3(), color::BLUE.to_vec3(), age_alpha));

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

    let top_left = (
        screen_space_position.x as i32,
        screen_space_position.y as i32,
    );

    if (top_left.0 as u32) > framebuffer.width - 8 {
        return;
    }

    if (top_left.1 as u32) > framebuffer.height - 8 {
        return;
    }

    if top_left.0 + 8 < 0 {
        return;
    }

    if top_left.1 + 8 < 0 {
        return;
    }

    Graphics::rectangle(
        framebuffer,
        top_left.0 as u32,
        top_left.1 as u32,
        8,
        8,
        Some(&color),
        None,
    )
}

use cairo::{buffer::Buffer2D, vec::vec3::Vec3};

pub static PIXELS_PER_METER: f32 = 16.0;

pub(crate) fn screen_to_world_space(screen_space_position: &Vec3, framebuffer: &Buffer2D) -> Vec3 {
    let mut world_space_position = *screen_space_position;

    world_space_position = (world_space_position - framebuffer.center) / PIXELS_PER_METER;

    world_space_position.y *= -1.0;

    world_space_position
}

pub(crate) fn world_to_screen_space(world_space_position: &Vec3, framebuffer: &Buffer2D) -> Vec3 {
    let mut screen_space_position = *world_space_position;

    screen_space_position.y *= -1.0;

    screen_space_position * PIXELS_PER_METER + framebuffer.center
}
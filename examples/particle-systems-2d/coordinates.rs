use cairo::vec::vec3::Vec3;

pub static PIXELS_PER_METER: f32 = 4.0;

pub(crate) fn screen_to_world_space(
    screen_space_position: &Vec3,
    framebuffer_center: &Vec3,
) -> Vec3 {
    let mut world_space_position = *screen_space_position;

    world_space_position = (world_space_position - framebuffer_center) / PIXELS_PER_METER;

    world_space_position.y *= -1.0;

    world_space_position
}

pub(crate) fn world_to_screen_space(
    world_space_position: &Vec3,
    framebuffer_center: &Vec3,
) -> Vec3 {
    let mut screen_space_position = *world_space_position;

    screen_space_position.y *= -1.0;

    screen_space_position * PIXELS_PER_METER + framebuffer_center
}

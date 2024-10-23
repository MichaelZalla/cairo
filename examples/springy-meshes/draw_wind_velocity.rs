use cairo::{buffer::Buffer2D, color, graphics::Graphics, vec::vec3::Vec3};

use crate::coordinates::world_to_screen_space;

pub(crate) fn draw_wind_velocity(
    wind: &Vec3,
    framebuffer: &mut Buffer2D,
    framebuffer_center: &Vec3,
) {
    let start_screen_space = world_to_screen_space(&Vec3::default(), framebuffer_center);
    let end_screen_space = world_to_screen_space(wind, framebuffer_center);

    let (x1, y1) = (start_screen_space.x as i32, start_screen_space.y as i32);
    let (x2, y2) = (end_screen_space.x as i32, end_screen_space.y as i32);

    Graphics::line(framebuffer, x1, y1, x2, y2, color::SKY_BOX.to_u32());
}

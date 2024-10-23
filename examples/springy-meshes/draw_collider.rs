use cairo::{buffer::Buffer2D, color, graphics::Graphics, vec::vec3::Vec3};

use crate::{
    coordinates::world_to_screen_space, static_line_segment_collider::StaticLineSegmentCollider,
};

pub(crate) fn draw_collider(
    collider: &StaticLineSegmentCollider,
    framebuffer: &mut Buffer2D,
    framebuffer_center: &Vec3,
) {
    let start_screen_space = world_to_screen_space(&collider.start, framebuffer_center);
    let end_screen_space = world_to_screen_space(&collider.end, framebuffer_center);

    let (x1, y1) = (start_screen_space.x as i32, start_screen_space.y as i32);
    let (x2, y2) = (end_screen_space.x as i32, end_screen_space.y as i32);

    Graphics::line(framebuffer, x1, y1, x2, y2, color::GREEN);

    // let midpoint_screen_space = world_to_screen_space(&collider.plane.point, framebuffer_center);
    // let normal_screen_space = world_to_screen_space(&collider.plane.normal, &Default::default());

    // let (x1, y1) = (
    //     midpoint_screen_space.x as i32,
    //     midpoint_screen_space.y as i32,
    // );

    // let (x2, y2) = (
    //     x1 + (normal_screen_space.x * PIXELS_PER_METER) as i32,
    //     y1 + (normal_screen_space.y * PIXELS_PER_METER) as i32,
    // );

    // Graphics::line(framebuffer, x1, y1, x2, y2, color::ORANGE);
}

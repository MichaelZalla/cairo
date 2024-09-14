use cairo::{
    buffer::Buffer2D,
    color::{self},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use crate::{coordinates::world_to_screen_space, springy_mesh::SpringyMesh};

pub(crate) fn draw_springy_mesh(
    mesh: &SpringyMesh,
    framebuffer: &mut Buffer2D,
    framebuffer_center: &Vec3,
) {
    static POINT_SIZE: u32 = 8;
    static POINT_SIZE_OVER_2: u32 = POINT_SIZE / 2;

    let points = &mesh.points;
    let struts = &mesh.struts;

    for point in points {
        let world_space_position = point.position;
        let screen_space_position =
            world_to_screen_space(&world_space_position, framebuffer_center);

        let x = screen_space_position.x as i32;
        let y = screen_space_position.y as i32;

        if let Some((x, y, width, height)) = Graphics::clip_rectangle(
            x - POINT_SIZE_OVER_2 as i32,
            y - POINT_SIZE_OVER_2 as i32,
            POINT_SIZE,
            POINT_SIZE,
            framebuffer,
        ) {
            Graphics::rectangle(framebuffer, x, y, width, height, Some(&color::YELLOW), None)
        }
    }

    for strut in struts {
        let start_world_space = points[strut.points.0].position;
        let start_screen_space = world_to_screen_space(&start_world_space, framebuffer_center);

        let end_world_space = points[strut.points.1].position;
        let end_screen_space = world_to_screen_space(&end_world_space, framebuffer_center);

        let (x1, y1) = (start_screen_space.x as i32, start_screen_space.y as i32);
        let (x2, y2) = (end_screen_space.x as i32, end_screen_space.y as i32);

        Graphics::line(framebuffer, x1, y1, x2, y2, &color::GREEN);
    }
}

use cairo::{
    animation::lerp,
    buffer::Buffer2D,
    color::{self, Color},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use crate::{
    coordinates::world_to_screen_space, renderable::Renderable, springy_mesh::SpringyMesh,
};

pub(crate) fn draw_springy_mesh(
    mesh: &SpringyMesh,
    framebuffer: &mut Buffer2D,
    framebuffer_center: &Vec3,
) {
    // Draw each point (vertex) as a square.

    for point in &mesh.points {
        point.render(framebuffer, framebuffer_center);
    }

    for strut in &mesh.struts {
        let start_world_space = &mesh.points[strut.points.0].position;
        let start_screen_space = world_to_screen_space(&start_world_space, framebuffer_center);

        let end_world_space = &mesh.points[strut.points.1].position;
        let end_screen_space = world_to_screen_space(&end_world_space, framebuffer_center);

        let (x1, y1) = (start_screen_space.x as i32, start_screen_space.y as i32);
        let (x2, y2) = (end_screen_space.x as i32, end_screen_space.y as i32);

        let elongation_alpha =
            ((strut.rest_length + strut.delta_length) / strut.rest_length / 2.0).clamp(0.0, 1.0);

        let color_vec3 = lerp(
            color::RED.to_vec3(),
            color::BLUE.to_vec3(),
            elongation_alpha,
        );

        let color = Color::from_vec3(color_vec3);

        Graphics::line(framebuffer, x1, y1, x2, y2, &color);
    }
}

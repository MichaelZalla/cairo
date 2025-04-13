use cairo::{buffer::Buffer2D, vec::vec3::Vec3};

pub trait Renderable {
    fn render(&self, buffer: &mut Buffer2D, buffer_center: &Vec3);
}

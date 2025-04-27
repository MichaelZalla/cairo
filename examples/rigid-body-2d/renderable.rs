use cairo::buffer::Buffer2D;

pub trait Renderable {
    fn render(&self, buffer: &mut Buffer2D);
}

use crate::buffer::Buffer2D;

pub trait Effect {
    fn apply(&self, buffer: &mut Buffer2D);
}

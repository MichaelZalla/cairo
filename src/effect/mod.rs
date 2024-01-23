use crate::buffer::Buffer2D;

pub mod kernel;

pub trait Effect {
    fn apply(&self, buffer: &mut Buffer2D);
}

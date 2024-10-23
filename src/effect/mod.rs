use std::{fmt::Debug, ops::Add};

use crate::buffer::Buffer2D;

pub trait Effect<T: Default + PartialEq + Copy + Clone + Debug + Add = u32> {
    fn apply(&mut self, buffer: &mut Buffer2D<T>);
}

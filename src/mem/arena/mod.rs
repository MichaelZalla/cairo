use error::AllocatorError;
use unique::UniquePtr;

pub mod error;
pub mod stack;
pub mod unique;

pub trait Arena {
    fn align(&self) -> usize;
    fn capacity(&self) -> usize;
    fn bytes_allocated(&self) -> usize;

    fn push(&mut self, size: usize) -> UniquePtr<[u8]>;
    fn push_zero(&mut self, size: usize) -> UniquePtr<[u8]>;

    fn push_array<T: Sized + Default + Clone>(&mut self, count: usize) -> UniquePtr<[T]>;

    fn push_for<T: Sized + Default + Clone>(&mut self) -> UniquePtr<T>;

    fn pop(&mut self, size: usize) -> Result<(), AllocatorError>;

    fn clear(&mut self);

    fn release(allocator: Self);
}

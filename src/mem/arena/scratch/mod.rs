use core::slice;
use std::ptr::NonNull;

use super::{error::AllocatorError, unique::UniquePtr, Arena};

pub struct ScratchArena<'a, T: Arena> {
    arena: &'a mut T,
    start: NonNull<u8>,
}

impl<'a, T: Arena> ScratchArena<'a, T> {
    pub fn new(arena: &'a mut T) -> Self {
        let start = {
            let borrowed = &arena;

            *borrowed.top()
        };

        Self { arena, start }
    }

    pub fn release(scratch: Self) -> Result<(), AllocatorError> {
        let slice = unsafe { slice::from_raw_parts_mut(scratch.start.as_ptr(), 0) };
        let start_slice = NonNull::new(slice).unwrap();

        scratch.arena.pop_to(UniquePtr::new(start_slice))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::arena::stack::FixedStackArena;

    #[test]
    fn test_new() {
        let mut stack = match FixedStackArena::new(1024, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        assert_eq!(stack.bytes_allocated(), 0);

        stack.push(1);
        stack.push(1);
        stack.push(1);

        assert_eq!(stack.bytes_allocated(), 3);

        let scratch = ScratchArena::new(&mut stack);

        assert_eq!(scratch.arena.bytes_allocated(), 3);
    }

    #[test]
    fn test_push_and_release() {
        let mut stack = match FixedStackArena::new(1024, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let _a = stack.push(1);
        let _b = stack.push(1);
        let c = stack.push(1);

        assert_eq!(stack.bytes_allocated(), 3);

        let scratch = ScratchArena::new(&mut stack);

        scratch.arena.push(1);
        scratch.arena.push(1);
        scratch.arena.push(1);

        assert_eq!(scratch.arena.bytes_allocated(), 6);

        assert!(ScratchArena::release(scratch).is_ok());

        assert_eq!(stack.bytes_allocated(), 3);

        let top = stack.top();

        assert!(unsafe { *top.as_ptr().byte_sub(1) == *(c.as_ptr().cast::<u8>()) });
    }
}

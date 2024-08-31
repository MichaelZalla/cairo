use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    ptr::NonNull,
};

use super::{error::AllocatorError, unique::UniquePtr, Arena};

pub struct FixedStackArena {
    capacity: usize,
    align: usize,
    layout: Layout,
    memory: UniquePtr<[u8]>,
    bytes_allocated: usize,
    finger: NonNull<u8>,
}

impl FixedStackArena {
    // Initialize an empty Arena.
    pub fn new(capacity: usize, align: usize) -> Result<Self, AllocatorError> {
        // Create layout.
        let layout = FixedStackArena::layout(capacity, align)?;

        let (memory, finger) = unsafe {
            // Allocate.
            let raw_ptr_mut = alloc(layout);
            // let raw_ptr_mut = libc::malloc(layout.size()).cast::<u8>();

            // Panic (stack unwind) or abort (no stack unwind) on OOM.
            if raw_ptr_mut.is_null() {
                handle_alloc_error(layout);
            }

            // Cast memory chunk to a slice.
            let slice = std::slice::from_raw_parts_mut(raw_ptr_mut, capacity);

            // Wrap slice inside a `UniquePtr`.
            let non_null = match NonNull::new(slice) {
                Some(ptr) => ptr,
                None => return Err(AllocatorError::Null),
            };

            // Finger pointer initialized to the start of our chunk.
            let finger = non_null.cast::<u8>();

            let unique_ptr = UniquePtr::new(non_null);

            (unique_ptr, finger)
        };

        Ok(Self {
            capacity,
            align,
            layout,
            memory,
            bytes_allocated: 0,
            finger,
        })
    }

    // Returns a Layout that corresponds to the given size and alignment.
    fn layout(size: usize, align: usize) -> Result<Layout, AllocatorError> {
        match Layout::from_size_align(size, align) {
            Ok(layout) => Ok(layout),
            Err(err) => Err(AllocatorError::Layout(err)),
        }
    }

    fn _push(&mut self, size: usize, zeroed: bool) -> UniquePtr<[u8]> {
        assert!(self.capacity() - self.bytes_allocated() >= size);

        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.finger.as_ptr(), size);

            if zeroed {
                slice.fill(0);
            }

            let non_null = match NonNull::new(slice) {
                Some(ptr) => ptr,
                None => panic!("{}", AllocatorError::Null),
            };

            let unique_ptr = UniquePtr::new(non_null);

            self.bytes_allocated += size;

            self.finger = NonNull::new_unchecked(self.finger.as_ptr().byte_add(size));

            unique_ptr
        }
    }

    fn _push_array<T: Sized + Default + Clone>(
        &mut self,
        count: usize,
        zeroed: bool,
    ) -> UniquePtr<[T]> {
        let unique = self._push(count * size_of::<T>(), false);

        let ptr_u8 = unique.as_ptr();

        let ptr_t = ptr_u8.cast::<T>();

        let slice_t = unsafe { std::slice::from_raw_parts_mut(ptr_t, count) };

        if zeroed {
            slice_t.fill(T::default())
        }

        match NonNull::new(slice_t) {
            Some(non_null) => UniquePtr::new(non_null),
            None => panic!("{}", AllocatorError::Null),
        }
    }
}

impl Arena for FixedStackArena {
    // Free the arena's underlying memory.
    fn release(mut allocator: Self) {
        allocator.clear();

        // allocator.finger = None;

        unsafe {
            let raw_ptr_mut = allocator.memory.as_ptr();

            let raw_ptr_mut_u8 = raw_ptr_mut.cast::<u8>();

            dealloc(raw_ptr_mut_u8, allocator.layout);
            // libc::free(raw_ptr_mut_u8.cast::<c_void>());
        }
    }

    fn bytes_allocated(&self) -> usize {
        self.bytes_allocated
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn align(&self) -> usize {
        self.align
    }

    // Push some bytes onto the stack.
    fn push(&mut self, size: usize) -> UniquePtr<[u8]> {
        self._push(size, false)
    }

    // Push some (zeroed) bytes onto the stack.
    fn push_zero(&mut self, size: usize) -> UniquePtr<[u8]> {
        self._push(size, true)
    }

    fn push_array<T: Sized + Default + Clone>(&mut self, count: usize) -> UniquePtr<[T]> {
        self._push_array(count, false)
    }

    fn push_for<T: Sized + Default + Clone>(&mut self) -> UniquePtr<T> {
        let unique = self._push_array::<T>(1, false);

        let ptr = unique.as_ptr().cast::<T>();

        match NonNull::new(ptr) {
            Some(non_null) => UniquePtr::new(non_null),
            None => panic!("{}", AllocatorError::Null),
        }
    }

    fn pop(&mut self, size: usize) -> Result<(), AllocatorError> {
        if size > self.bytes_allocated {
            return Err(AllocatorError::InvalidArguments);
        }

        self.bytes_allocated -= size;

        unsafe {
            let ptr = self.finger.as_ptr();

            self.finger = NonNull::new_unchecked(ptr.byte_sub(size));
        }

        Ok(())
    }

    fn pop_to(&mut self, position: UniquePtr<[u8]>) -> Result<(), AllocatorError> {
        let target = position.as_ptr().cast::<u8>();
        let start = self.memory.as_ptr().cast::<u8>();
        let end = self.finger.as_ptr();

        if target < start || target > end {
            return Err(AllocatorError::InvalidArguments);
        }

        unsafe {
            self.bytes_allocated = target.offset_from(start).try_into().unwrap();

            match NonNull::new(self.memory.as_ptr().byte_add(self.bytes_allocated)) {
                Some(non_null) => {
                    self.finger = non_null.cast::<u8>();
                }
                None => panic!(),
            }
        }

        Ok(())
    }

    fn clear(&mut self) {
        self.pop(self.bytes_allocated()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::Arena;
    use super::*;

    #[test]
    fn test_initialization() {
        let stack = match FixedStackArena::new(1024, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        assert_eq!(stack.capacity(), 1 << 10);
        assert_eq!(stack.align(), 1 << 0);
        assert_eq!(stack.bytes_allocated(), 0);
    }

    #[test]
    fn test_pop_on_empty_panic() {
        let mut stack = match FixedStackArena::new(1024, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        assert!(stack.pop(1).is_err())
    }

    #[test]
    fn test_push_and_pop() {
        let mut stack = match FixedStackArena::new(32, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 0);

        let _ = stack.push(1);

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 1);

        let _ = stack.push(1);

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 2);

        let _ = stack.push(1);

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 3);

        assert!(stack.pop(1).is_ok());

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 2);

        assert!(stack.pop(1).is_ok());

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 1);

        assert!(stack.pop(1).is_ok());

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 0);
    }

    #[test]
    fn test_push_zero() {
        let mut stack = match FixedStackArena::new(32, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let mut a = stack.push_zero(1);
        let mut b = stack.push_zero(1);
        let mut c = stack.push_zero(1);

        assert_eq!(*a.as_mut().first().unwrap(), 0);
        assert_eq!(*b.as_mut().first().unwrap(), 0);
        assert_eq!(*c.as_mut().first().unwrap(), 0);
    }

    #[test]
    fn test_mutable_memory() {
        let mut stack = match FixedStackArena::new(32, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let mut a = stack.push_zero(1);
        let mut b = stack.push_zero(1);
        let mut c = stack.push_zero(1);
        let mut d = stack.push_zero(1);

        *a.as_mut().first_mut().unwrap() = 1;
        *b.as_mut().first_mut().unwrap() = 2;
        *c.as_mut().first_mut().unwrap() = 3;
        *d.as_mut().first_mut().unwrap() = 4;

        assert_eq!(*a.as_mut().first().unwrap(), 1);
        assert_eq!(*b.as_mut().first().unwrap(), 2);
        assert_eq!(*c.as_mut().first().unwrap(), 3);
        assert_eq!(*d.as_mut().first().unwrap(), 4);
    }

    #[test]
    fn test_push_array() {
        let mut stack = match FixedStackArena::new(128, 4) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        assert_eq!(stack.capacity(), 128);
        assert_eq!(stack.align(), 4);
        assert_eq!(stack.bytes_allocated(), 0);

        let count = 4;

        let mut floats = stack.push_array::<f32>(count);

        assert_eq!(stack.capacity(), 128);
        assert_eq!(stack.align(), 4);
        assert_eq!(stack.bytes_allocated(), count * size_of::<f32>());

        floats.as_mut()[0] = 1.1;
        floats.as_mut()[1] = 2.2;
        floats.as_mut()[2] = 3.3;
        floats.as_mut()[3] = 4.4;

        stack.clear();

        let floats = stack.push_array::<f32>(count);

        assert_eq!(stack.capacity(), 128);
        assert_eq!(stack.align(), size_of::<f32>());
        assert_eq!(stack.bytes_allocated(), count * size_of::<f32>());

        assert_eq!(floats.as_ref()[0], 1.1);
        assert_eq!(floats.as_ref()[1], 2.2);
        assert_eq!(floats.as_ref()[2], 3.3);
        assert_eq!(floats.as_ref()[3], 4.4);
    }

    #[test]
    fn test_push_in() {
        let mut stack = match FixedStackArena::new(128, size_of::<f32>()) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let mut data = stack.push_for::<f32>();

        println!("Before: {}", *data);

        *data = 3.14;

        println!("After: {}", *data);
    }

    #[test]
    fn test_pop_to() {
        let mut stack = match FixedStackArena::new(32, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let a = stack.push(1);
        let b = stack.push(1);
        let _c = stack.push(1);
        let e = stack.push(1);
        let _f = stack.push(1);
        let _g = stack.push(1);

        assert!(stack.pop_to(e).is_ok());
        assert_eq!(stack.bytes_allocated(), 3);

        assert!(stack.pop_to(b).is_ok());
        assert_eq!(stack.bytes_allocated(), 1);

        assert!(stack.pop_to(a).is_ok());
        assert_eq!(stack.bytes_allocated(), 0);
    }

    #[test]
    fn test_clear() {
        let mut stack = match FixedStackArena::new(32, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let _ = stack.push_zero(1);
        let _ = stack.push_zero(1);
        let _ = stack.push_zero(1);

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 3);

        stack.clear();

        assert_eq!(stack.capacity(), 32);
        assert_eq!(stack.bytes_allocated(), 0);
    }

    #[test]
    fn test_release() {
        let mut stack = match FixedStackArena::new(16_000_000_000, 1) {
            Ok(stack) => stack,
            Err(err) => panic!("{}", err.to_string()),
        };

        let _a = stack.push_zero(1);
        let _b = stack.push_zero(1);
        let _c = stack.push_zero(1);

        assert_eq!(stack.capacity(), 16_000_000_000);
        assert_eq!(stack.bytes_allocated(), 3);

        FixedStackArena::release(stack);

        // Note: `FixedStackAllocator::release()` consumes `Self`, so we can't
        // assert on `stack` here.
        // assert_eq!(stack.capacity(), 0);
        // assert_eq!(stack.bytes_allocated(), 0);

        // let ptr = c.as_ptr().cast::<u8>();

        // unsafe {
        //     // Used to test for a use-after-free page fault, for large arenas.

        //     // By including this code, you can demonstrate that a fault occurs,
        //     // but verifying this without crashing your test runner is
        //     // difficult—so, here, we'll just comment it out.

        //     // std::ptr::write_volatile(ptr, 255);
        // }
    }
}

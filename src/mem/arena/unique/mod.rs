use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct UniquePtr<T: ?Sized> {
    pointer: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> UniquePtr<T> {
    pub fn new(pointer: NonNull<T>) -> Self {
        Self {
            pointer,
            _marker: PhantomData,
        }
    }
}

impl<T: Sized> Deref for UniquePtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Sized> DerefMut for UniquePtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: ?Sized> UniquePtr<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }

    pub fn as_ref<'a>(&self) -> &'a T {
        unsafe { self.pointer.as_ref() }
    }

    pub fn as_mut<'a>(&mut self) -> &'a mut T {
        unsafe { self.pointer.as_mut() }
    }
}

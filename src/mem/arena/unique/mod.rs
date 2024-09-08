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
}

impl<T: ?Sized> AsRef<T> for UniquePtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T: ?Sized> AsMut<T> for UniquePtr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }
}

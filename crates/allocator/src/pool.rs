use std::{
    ops::{Deref, DerefMut},
    slice,
};

use crate::backend::{alloc_standard, dealloc_standard};
use error::Error::{self};

struct Element<'a, T: Sized> {
    pub element: T,
    pub next: *mut Element<'a, T>,
}

pub struct PoolAllocator<'a, T: Sized> {
    arr: &'a mut [Element<'a, T>],
    head: *mut Element<'a, T>,
    len: usize,
    used: usize,
    next: Option<Box<PoolAllocator<'a, T>>>,
}

impl<'a, T: Sized> PoolAllocator<'a, T> {
    pub fn from_size(len: usize) -> Result<Self, Error> {
        let ptr =
            unsafe { alloc_standard(len * size_of::<Element<T>>(), align_of::<Element<T>>())? };
        unsafe {
            Ok(Self {
                arr: slice::from_raw_parts_mut(ptr as *mut Element<T>, len),
                head: ptr as *mut Element<'a, T>,
                len,
                used: 0,
                next: None,
            })
        }
    }

    pub fn alloc(&mut self) -> Result<&'a mut T, Error> {
        if self.used == self.len {
            let next = self
                .next
                .get_or_insert(Box::new(Self::from_size(self.len)?));
            let tmp: *mut Element<T> = next.head;
            next.head = unsafe { (*next.head).next };
            next.used += 1;
            Ok(unsafe { &mut (*tmp).element })
        } else {
            let tmp: *mut Element<T> = self.head;
            self.head = unsafe { (*self.head).next };
            self.used += 1;
            Ok(unsafe { &mut (*tmp).element })
        }
    }

    pub fn dealloc(&mut self, target: &'a mut T) -> Result<(), Error> {
        let obj: &mut PoolAllocator<'a, T> = if self.used == self.len {
            unsafe { self.next.as_mut().unwrap_unchecked().as_mut() }
        } else {
            self
        };

        let next = unsafe { (*obj.head).next };
        obj.head = target as *mut T as *mut Element<'a, T>;
        unsafe {
            (*obj.head).next = next;
        };
        Ok(())
    }
}

impl<'a, T: Sized> Drop for PoolAllocator<'a, T> {
    fn drop(&mut self) {
        unsafe {
            dealloc_standard(self.arr.as_mut_ptr(), self.len * size_of::<Element<T>>())
                .expect("failed to deallocate memory")
        };
    }
}

impl<T> AsRef<T> for Element<'_, T> {
    fn as_ref(&self) -> &T {
        &self.element
    }
}

impl<T> AsMut<T> for Element<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.element
    }
}

impl<T> Deref for Element<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl<T> DerefMut for Element<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.element
    }
}

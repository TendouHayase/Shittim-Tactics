use std::{
    cell::Cell,
    hash::Hash,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use crate::backend::{alloc_standard, dealloc_standard};
use error::Error::{self, MemoryAllocateFailed};

#[repr(C)]
#[derive(Debug)]
pub struct Element<'a, T: Sized> {
    element: T,
    next: *mut Element<'a, T>,
}

pub struct ElementGuard<'a, T: Sized> {
    elem: NonNull<Element<'a, T>>,
    pool: *mut PoolAllocator<'a, T>,
    _marker: std::marker::PhantomData<&'a mut T>,
}

pub struct PoolAllocator<'a, T: Sized> {
    arr: &'a mut [Element<'a, T>],
    head: Cell<*mut Element<'a, T>>,
    len: usize,
    used: usize,
}

impl<'a, T: Sized> PartialEq for ElementGuard<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.elem == other.elem
    }
}

impl<T: Sized> Eq for ElementGuard<'_, T> {}

impl<'a, T: Sized> Deref for ElementGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &self.elem.as_ref().element }
    }
}

impl<T: Sized + Hash> Hash for ElementGuard<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.elem.hash(state)
    }
}

impl<'a, T: Sized> DerefMut for ElementGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut self.elem.as_mut().element }
    }
}

impl<'a, T: Sized> Drop for ElementGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let pool = &mut *self.pool;
            let head = pool.head.get_mut();
            // Set the `next` of `elem` to the existing `head`
            self.elem.as_mut().next = *head;
            // Update `free_head` to the current `elem`
            *head = self.elem.as_ptr();

            (*self.pool).used -= 1;
        }
    }
}

impl<'a, T: Sized> PoolAllocator<'a, T> {
    pub fn from_size(pool_size: usize) -> Result<Self, Error> {
        if pool_size.overflowing_mul(size_of::<Element<T>>()).1 {
            return Err(MemoryAllocateFailed(
                "Attempt to allocate too much memory".to_string(),
            ));
        }
        let ptr = unsafe {
            alloc_standard(
                pool_size * size_of::<Element<T>>(),
                align_of::<Element<T>>(),
            )?
        };
        let result = unsafe {
            Self {
                arr: slice::from_raw_parts_mut(ptr as *mut Element<T>, pool_size),
                head: Cell::from(ptr as *mut Element<'_, T>),
                len: pool_size,
                used: 0,
            }
        };
        result.arr.iter_mut().enumerate().for_each(|item| {
            item.1.next = if item.0 != pool_size - 1 {
                (item.1 as *mut Element<T> as usize + size_of::<Element<'a, T>>())
                    as *mut Element<T>
            } else {
                std::ptr::null_mut()
            };
        });

        Ok(result)
    }

    pub fn alloc(&mut self) -> Result<ElementGuard<'a, T>, Error> {
        if self.used == self.len {
            return Err(MemoryAllocateFailed("Memory Pool is full".to_string()));
        }
        let head = self.head.get_mut();

        self.used += 1;

        unsafe {
            let elem_ptr = *head;
            *head = (*elem_ptr).next;
            Ok(ElementGuard {
                elem: NonNull::new_unchecked(elem_ptr),
                pool: self as *mut Self,
                _marker: std::marker::PhantomData,
            })
        }
    }

    pub fn dealloc(&mut self, guard: ElementGuard<'a, T>) {
        drop(guard);
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

impl<T: Sized + PartialEq> PartialEq for Element<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element && self.next == other.next
    }
}

impl<T: Sized + Hash> Hash for Element<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.element.hash(state);
    }
}

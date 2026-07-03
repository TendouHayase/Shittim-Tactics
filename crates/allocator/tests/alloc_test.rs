use std::{alloc::Layout, task::Poll};

use allocator::pool::PoolAllocator;

#[derive(Debug, PartialEq)]
struct Tracked {
    tag: u32,
}

#[test]
fn from_size_creates_pool_of_given_capacity() {
    let pool = PoolAllocator::<Tracked>::from_size(4);
    assert!(pool.is_ok());
}

#[test]
fn from_size_zero_should_empty() {
    let result = PoolAllocator::<Tracked>::from_size(0);
    match result {
        Ok(mut pool) => {
            assert!(pool.alloc().is_err());
        }
        Err(_) => {
            // len=0을 명시적으로 에러 처리하는 구현도 유효함
        }
    }
}

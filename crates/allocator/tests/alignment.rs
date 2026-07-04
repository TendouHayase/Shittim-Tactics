use std::mem::{align_of, size_of};

use allocator::pool::PoolAllocator;

#[test]
fn alloc_alignment_u8() {
    let mut pool: PoolAllocator<u8> = PoolAllocator::from_size(32).expect("pool creation failed");
    let align = align_of::<u8>();
    for _ in 0..32 {
        let slot = pool.alloc().expect("alloc should succeed");
        let addr = *slot as *const u8 as usize;
        assert_eq!(
            addr % align,
            0,
            "u8 addr {:#x} not aligned to {}",
            addr,
            align
        );
    }
}

#[test]
fn alloc_alignment_u64() {
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(32).expect("pool creation failed");
    let align = align_of::<u64>();
    for _ in 0..32 {
        let slot = pool.alloc().expect("alloc should succeed");
        let addr = *slot as *const u64 as usize;
        assert_eq!(
            addr % align,
            0,
            "u64 addr {:#x} not aligned to {}",
            addr,
            align
        );
    }
}

#[test]
fn alloc_alignment_u128() {
    let mut pool: PoolAllocator<u128> = PoolAllocator::from_size(16).expect("pool creation failed");
    let align = align_of::<u128>();
    for _ in 0..16 {
        let slot = pool.alloc().expect("alloc should succeed");
        let addr = *slot as *const u128 as usize;
        assert_eq!(
            addr % align,
            0,
            "u128 addr {:#x} not aligned to {}",
            addr,
            align
        );
    }
}

#[test]
fn alloc_alignment_struct_with_align() {
    #[repr(C, align(64))]
    struct Aligned64 {
        _data: [u8; 64],
    }

    let mut pool: PoolAllocator<Aligned64> =
        PoolAllocator::from_size(8).expect("pool creation failed");
    for _ in 0..8 {
        let slot = pool.alloc().expect("alloc should succeed");
        let addr = &*slot as *const Aligned64 as usize;
        assert_eq!(addr % 64, 0, "Aligned64 addr {:#x} not aligned to 64", addr);
    }
}

#[test]
fn alloc_size_matches_type_size() {
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(2).expect("pool creation failed");
    let s0 = pool.alloc().expect("alloc 0");
    let s1 = pool.alloc().expect("alloc 1");

    let addr0 = *s0 as *const u64 as usize;
    let addr1 = *s1 as *const u64 as usize;
    let diff = (addr1 as isize - addr0 as isize).unsigned_abs();

    // 두 연속 할당의 주소 차이는 최소 size_of::<u64>()여야 함
    assert!(
        diff >= size_of::<u64>(),
        "slot distance {} < sizeof(u64)={}",
        diff,
        size_of::<u64>()
    );
}

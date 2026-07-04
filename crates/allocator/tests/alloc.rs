use allocator::pool::{ElementGuard, PoolAllocator};

#[test]
fn alloc_exact_capacity_all_succeed() {
    const LEN: usize = 64;
    let mut pool: PoolAllocator<u32> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    let mut ptrs = Vec::new();
    for i in 0..LEN {
        let mut slot = pool.alloc().expect(&format!("alloc #{} should succeed", i));
        *slot = i as u32;
        ptrs.push(*slot as *const u32 as usize);
    }
    assert_eq!(ptrs.len(), LEN);
}

#[test]
fn alloc_beyond_capacity_returns_err() {
    const LEN: usize = 8;
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(LEN).expect("pool creation failed");
    let mut slots: Vec<ElementGuard<u64>> = Vec::new();

    for _ in 0..LEN {
        let g = pool.alloc().expect("alloc within capacity should succeed");
        slots.push(g);
    }
    // 풀이 가득 찬 상태
    let result = pool.alloc();
    assert!(result.is_err(), "alloc beyond capacity should return Err");
}

#[test]
fn alloc_returns_distinct_addresses() {
    const LEN: usize = 32;
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    let mut addrs = std::collections::HashSet::new();
    for i in 0..LEN {
        let mut slot = pool.alloc().expect("alloc should succeed");
        *slot = i as u64;
        assert!(addrs.insert(slot), "duplicate address returned : {i}");
    }
}

#[test]
fn alloc_returns_aligned_addresses() {
    const LEN: usize = 16;
    let mut pool: PoolAllocator<u128> =
        PoolAllocator::from_size(LEN).expect("pool creation failed");
    let align = std::mem::align_of::<u128>();

    for _ in 0..LEN {
        let slot = pool.alloc().expect("alloc should succeed");
        let addr = *slot as *const u128 as usize;
        assert_eq!(
            addr % align,
            0,
            "address {:#x} not aligned to {}",
            addr,
            align
        );
    }
}

#[test]
fn alloc_writable_and_readable() {
    let mut pool: PoolAllocator<u32> = PoolAllocator::from_size(4).expect("pool creation failed");

    let mut slot = pool.alloc().expect("alloc should succeed");
    *slot = 0xDEAD_BEEF;
    assert_eq!(*slot, 0xDEAD_BEEF);
}

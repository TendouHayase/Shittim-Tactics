use allocator::pool::PoolAllocator;

#[test]
fn dealloc_then_alloc_reuses_slot() {
    const LEN: usize = 4;
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    // 풀을 가득 채운다
    let s0 = pool.alloc().expect("alloc 0");
    let s1 = pool.alloc().expect("alloc 1");
    let s2 = pool.alloc().expect("alloc 2");
    let s3 = pool.alloc().expect("alloc 3");

    let addr_before = *s1 as *const u64 as usize;

    // s1을 해제
    pool.dealloc(s1);

    // 다시 alloc하면 해제된 슬롯이 재사용되어야 함 (LIFO라면 같은 주소)
    let s_reuse = pool.alloc().expect("alloc after dealloc should succeed");
    let addr_after = *s_reuse as *const u64 as usize;

    assert_eq!(
        addr_before, addr_after,
        "reused slot should have same address"
    );

    // 사용하지 않은 슬롯 정리 (컴파일을 위한 dummy 사용)
    let _ = (s0, s2, s3);
}

#[test]
fn dealloc_frees_capacity() {
    const LEN: usize = 2;
    let mut pool: PoolAllocator<u32> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    let s0 = pool.alloc().expect("alloc 0");
    let s1 = pool.alloc().expect("alloc 1");

    // 풀 가득 참
    assert!(pool.alloc().is_err(), "pool should be full");

    // 하나 해제
    pool.dealloc(s0);

    // 다시 하나 할당 가능해야 함
    let s2 = pool.alloc().expect("alloc after dealloc should succeed");
    let _ = (s1, s2);
}

#[test]
fn dealloc_all_then_realloc_all() {
    const LEN: usize = 16;
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    // 전부 alloc
    let mut slots = Vec::new();
    for _ in 0..LEN {
        slots.push(pool.alloc().expect("alloc should succeed"));
    }

    // 역순으로 dealloc
    while let Some(slot) = slots.pop() {
        pool.dealloc(slot);
    }

    // 다시 전부 alloc 가능해야 함
    let mut count = 0;
    for _ in 0..LEN {
        let _slot = pool.alloc().expect("realloc should succeed");
        count += 1;
    }
    assert_eq!(count, LEN);
}

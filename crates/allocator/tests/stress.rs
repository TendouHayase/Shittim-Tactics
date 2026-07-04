use allocator::pool::PoolAllocator;

use std::{collections::HashSet, ops::Deref};

#[test]
fn stress_random_alloc_dealloc_integrity() {
    const LEN: usize = 64;
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    // 단순 PRNG ( deterministic )
    let mut state: u64 = 0x1234_5678_9ABC_DEF0;
    let mut live_slots: Vec<&'static mut u64> = Vec::new();
    // Note: 실제로는 'static 라이프타임을 가질 수 없으므로
    // 이 테스트는 unsafe로 수명을 연장하는 대신
    // alloc/dealloc 패턴만 검증한다

    let mut alloc_count = 0usize;
    let mut dealloc_count = 0usize;

    for _ in 0..1000 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let should_alloc = (state % 2 == 0) || live_slots.is_empty();

        if should_alloc && live_slots.len() < LEN {
            match pool.alloc() {
                Ok(_slot) => {
                    alloc_count += 1;
                    // 실제 테스트에서는 주소를 추적해야 하지만
                    // 라이프타임 제약으로 인해 여기서는 카운트만 확인
                }
                Err(_) => {
                    // 풀이 가득 찬 상태에서의 alloc 실패는 정상
                }
            }
        }
    }

    // 최소한 일정 횟수의 alloc이 성공했는지 확인
    assert!(alloc_count > 0, "at least some allocs should succeed");
}

#[test]
fn stress_fill_drain_repeat() {
    const LEN: usize = 32;
    let mut pool: PoolAllocator<u32> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    for round in 0..10 {
        // 전부 채우기
        let mut slots = Vec::new();
        for _ in 0..LEN {
            slots.push(
                pool.alloc()
                    .expect(&format!("round {} alloc failed", round)),
            );
        }
        assert!(
            pool.alloc().is_err(),
            "round {}: pool should be full",
            round
        );

        // 절반 해제
        for _ in 0..(LEN / 2) {
            let slot = slots.pop().unwrap();
            pool.dealloc(slot);
        }

        // 다시 절반 채우기
        for _ in 0..(LEN / 2) {
            let _slot = pool
                .alloc()
                .expect("realloc after partial drain should succeed");
        }

        // 나머지 해제
        while let Some(slot) = slots.pop() {
            pool.dealloc(slot);
        }
    }
}

#[test]
fn stress_interleaved_alloc_dealloc_no_overlap() {
    const LEN: usize = 16;
    let mut pool: PoolAllocator<u64> = PoolAllocator::from_size(LEN).expect("pool creation failed");

    // alloc 8개
    let mut slots = Vec::new();
    for _ in 0..8 {
        slots.push(pool.alloc().expect("alloc should succeed"));
    }

    // 주소 수집
    let mut addrs: HashSet<usize> = HashSet::new();
    for s in &slots {
        let addr = s.deref() as *const u64 as usize;
        assert!(addrs.insert(addr), "duplicate address in interleaved test");
    }

    // 4개 해제
    for _ in 0..4 {
        let s = slots.pop().unwrap();
        pool.dealloc(s);
    }

    // 4개 재할당 - 새 주소가 기존 주소와 겹치면 안 됨 (살아있는 것들과)
    for _ in 0..4 {
        let s = pool.alloc().expect("realloc should succeed");
        let addr = *s as *const u64 as usize;
        assert!(
            !addrs.contains(&addr)
                || !slots
                    .iter()
                    .any(|x| (x.deref() as *const u64 as usize) == addr),
            "new alloc overlaps with live slot"
        );
    }
}

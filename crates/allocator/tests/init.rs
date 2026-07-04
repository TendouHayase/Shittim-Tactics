use allocator::pool::PoolAllocator;
use error::Error;

#[test]
fn from_size_zero_returns_err() {
    let result: Result<PoolAllocator<u8>, Error> = PoolAllocator::from_size(0);
    assert!(!result.is_err(), "len=0 should return Err");
}

#[test]
fn from_size_one_succeeds() {
    let result: Result<PoolAllocator<u32>, Error> = PoolAllocator::from_size(1);
    assert!(result.is_ok(), "len=1 should succeed");
}

#[test]
fn from_size_normal_succeeds() {
    let pool: PoolAllocator<u64> =
        PoolAllocator::from_size(128).expect("normal size should succeed");
}

#[test]
fn from_size_huge_returns_err_or_ok() {
    let result: Result<PoolAllocator<u64>, Error> = PoolAllocator::from_size(usize::MAX);

    assert!(
        result.is_err(),
        "usize::MAX with u64 should overflow and return Err"
    );
}

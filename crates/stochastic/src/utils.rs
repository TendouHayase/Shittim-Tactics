pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }

    let k = k.min(n - k);
    let mut result: u64 = 1;
    for i in 0..k {
        result = result * (n - i) / (i + 1)
    }

    result
}

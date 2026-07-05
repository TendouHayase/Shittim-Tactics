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

pub fn factorial(n: u32) -> u64 {
    (1..=n as u64).product()
}

pub fn build_prefix_sum(counts: &[u128]) -> Vec<u128> {
    let mut prefix = vec![0u128; counts.len() + 1];
    for i in 0..counts.len() {
        prefix[i + 1] = prefix[i] + counts[i];
    }
    prefix
}

pub mod composite;
pub mod distributions;
pub mod range;
pub mod utils;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        composite::Composite,
        distributions::{IrwinHall, Normal, Uniform},
        range::RangeProbability,
        utils::build_prefix_sum,
    };

    fn make_base(min: u64, max: u64) -> IrwinHall {
        let counts = vec![1u128; (max - min + 1) as usize];
        let prefix = build_prefix_sum(&counts);
        IrwinHall {
            prefix_sum: Arc::new(prefix),
            uniforms: Arc::new(RwLock::new(vec![Uniform { min, max }])),
            n: 1,
            min,
            max,
            total_combinations: (max - min + 1) as u128,
        }
    }

    // ==================== Uniform ====================

    #[test]
    fn uniform_full_range_is_one() {
        let u = Uniform::new(0, 9);
        let prob = u.range_probability(0.0, 9.0);
        assert!((prob - 1.0).abs() < 1e-6);
    }

    #[test]
    fn uniform_half_range() {
        let u = Uniform::new(0, 9);
        let prob = u.range_probability(0.0, 4.0);
        assert!((prob - 0.5).abs() < 1e-6);
    }

    #[test]
    fn uniform_subrange_ratio() {
        // [4,6] = 2 units, [2,10] = 8 units → 0.25
        let u = Uniform::new(1, 8);
        let prob = u.range_probability(4.0, 5.0);
        assert!((prob - 0.25).abs() < 1e-6);
    }

    #[test]
    fn uniform_zero_width() {
        let u = Uniform::new(0, 9);
        let prob = u.range_probability(3.0, 3.0);
        assert!((prob - 0.1f64).abs() < 1e-6);
    }

    #[test]
    fn uniform_outside_support() {
        let u = Uniform::new(0, 9);
        let prob = u.range_probability(20.0, 30.0);
        assert!(prob.abs() < 1e-6);
    }

    #[test]
    fn uniform_partial_overlap() {
        // [5,15] ∩ [0,10] = [5,10] → 5/10 = 0.5
        let u = Uniform::new(0, 9);
        let prob = u.range_probability(5.0, 15.0);
        assert!((prob - 0.5).abs() < 1e-6);
    }

    #[test]
    fn uniform_reversed_range() {
        let u = Uniform::new(0, 9);
        // a > b 인 경우 0 또는 에러
        let result = u.range_probability(8.0, 2.0);
        assert!(result.abs() < 1e-6, "reversed range should be 0");
    }

    #[test]
    fn uniform_additivity() {
        // P(a,c) = P(a,b) + P(b,c)
        let u = Uniform::new(0, 9);
        let p_ac = u.range_probability(2.0, 8.0);
        let p_bc = u.range_probability(5.0, 8.0);
        let p_ab = u.range_probability(2.0, 4.0);
        assert!((p_ac - (p_ab + p_bc)).abs() < 1e-6);
    }

    // ==================== Normal ====================

    #[test]
    fn normal_wide_range_approx_one() {
        let n = Normal::new(0.0, 1.0);
        let prob = n.range_probability(-100.0, 100.0);
        assert!((prob - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normal_one_sigma() {
        // P(-1σ, 1σ) ≈ 0.6827
        let n = Normal::new(0.0, 1.0);
        let prob = n.range_probability(-1.0, 1.0);
        assert!(
            (prob - 0.6826894921370859).abs() < 1e-6,
            "expected ≈0.6827, got {}",
            prob
        );
    }

    #[test]
    fn normal_two_sigma() {
        // P(-2σ, 2σ) ≈ 0.9545
        let n = Normal::new(0.0, 1.0);
        let prob = n.range_probability(-2.0, 2.0);
        assert!(
            (prob - 0.9544997361036416).abs() < 1e-6,
            "expected ≈0.9545, got {}",
            prob
        );
    }

    #[test]
    fn normal_three_sigma() {
        // P(-3σ, 3σ) ≈ 0.9973
        let n = Normal::new(0.0, 1.0);
        let prob = n.range_probability(-3.0, 3.0);
        assert!(
            (prob - 0.9973002039367398).abs() < 1e-6,
            "expected ≈0.9973, got {}",
            prob
        );
    }

    #[test]
    fn normal_symmetry_around_mean() {
        let n = Normal::new(5.0, 2.0);
        let left = n.range_probability(1.0, 5.0);
        let right = n.range_probability(5.0, 9.0);
        assert!((left - right).abs() < 1e-6);
    }

    #[test]
    fn normal_shift_invariance() {
        // N(10, 2)의 P(8, 12) == N(0,1)의 P(-1, 1)
        let n = Normal::new(10.0, 2.0);
        let std_n = Normal::new(0.0, 1.0);
        let prob = n.range_probability(8.0, 12.0);
        let std_prob = std_n.range_probability(-1.0, 1.0);
        assert!((prob - std_prob).abs() < 1e-6);
    }

    #[test]
    fn normal_zero_width() {
        let n = Normal::new(0.0, 1.0);
        let prob = n.range_probability(0.0, 0.0);
        assert!(prob.abs() < 1e-6);
    }

    #[test]
    fn normal_additivity() {
        let n = Normal::new(0.0, 1.0);
        let p_ac = n.range_probability(-2.0, 2.0);
        let p_ab = n.range_probability(-2.0, 0.0);
        let p_bc = n.range_probability(0.0, 2.0);
        assert!((p_ac - (p_ab + p_bc)).abs() < 1e-6);
    }

    #[test]
    fn normal_monotone_increasing() {
        // CDF는 단조 증가 → 구간이 넓어지면 확률도 커져야 함
        let n = Normal::new(0.0, 1.0);
        let narrow = n.range_probability(-0.5, 0.5);
        let wide = n.range_probability(-1.0, 1.0);
        assert!(wide > narrow);
    }

    // ==================== Composite ====================

    #[test]
    fn test_compose_does_not_mutate_self() {
        let base = make_base(0, 5);
        let original_min = base.min;
        let original_max = base.max;
        let original_total = base.total_combinations;

        let rhs = Uniform { min: 2, max: 8 };
        let composed = base.compose(&rhs);

        // self는 그대로 유지되어야 함
        assert_eq!(base.min, original_min);
        assert_eq!(base.max, original_max);
        assert_eq!(base.total_combinations, original_total);

        // 새 인스턴스는 값이 갱신되어야 함
        assert_eq!(composed.min, 0 + 2);
        assert_eq!(composed.max, 5 + 8);
        assert_eq!(composed.total_combinations, 6 * 7);
        assert_eq!(composed.n, 2);
    }

    #[test]
    fn test_compose_chain_matches_brute_force() {
        let base = make_base(0, 5);
        let composed = base
            .compose(&Uniform { min: 2, max: 8 })
            .compose(&Uniform { min: 1, max: 3 });

        // 브루트포스: 0..=5, 2..=8, 1..=3 세 범위의 모든 조합 합산
        let mut total_ways = 0u128;
        let mut counts_by_value = std::collections::HashMap::new();
        for x in 0..=5u64 {
            for y in 2..=8u64 {
                for z in 1..=3u64 {
                    let s = x + y + z;
                    *counts_by_value.entry(s).or_insert(0u128) += 1;
                    total_ways += 1;
                }
            }
        }

        assert_eq!(composed.total_combinations, total_ways);

        let test_cases = [(0u64, 5u64), (6, 10), (10, 16), (3, 8)];
        for (start, end) in test_cases {
            let expected: u128 = counts_by_value
                .iter()
                .filter(|(val, _)| **val >= start && **val <= end)
                .map(|(_, &cnt)| cnt)
                .sum();
            let expected_prob = expected as f64 / total_ways as f64;

            let actual = composed.range_probability(start, end);
            assert!(
                (actual - expected_prob).abs() < 1e-12,
                "range=({start},{end}): expected={expected_prob}, actual={actual}"
            );
        }
    }
}

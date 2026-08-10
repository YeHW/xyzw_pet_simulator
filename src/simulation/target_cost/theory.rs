use crate::game::{P, pity_threshold};

// 无保底理论期望（精确）
// C_1 = 1
// C_n = C_(n-1) * (1 + p(n-1)) / p(n-1)
pub fn theoretical_cost_no_pity(target: usize) -> f64 {
    assert!((1..=7).contains(&target), "target 必须在 1..=7");

    let mut c = 1.0;
    for level in 1..target {
        let p = P[level];
        c *= (1.0 + p) / p;
    }
    c
}

fn layer_expected_consumption_for_outputs(level: usize, outputs: usize) -> f64 {
    if outputs == 0 {
        return 0.0;
    }

    let p = P[level];
    let q = 1.0 - p;
    let r = pity_threshold(level + 1);

    let mut next = vec![0.0_f64; r];

    for _o in (0..outputs).rev() {
        let mut curr = vec![0.0_f64; r];

        for b in (0..r).rev() {
            let success_cost = 2.0 + next[b];
            let fail_cost = if b + 1 == r {
                1.0 + next[0]
            } else {
                1.0 + curr[b + 1]
            };

            curr[b] = p * success_cost + q * fail_cost;
        }

        next = curr;
    }

    next[0]
}

fn layer_expected_consumption_for_fractional_outputs(level: usize, outputs: f64) -> f64 {
    if outputs <= 0.0 {
        return 0.0;
    }

    let low = outputs.floor() as usize;
    let frac = outputs - low as f64;

    if frac == 0.0 {
        return layer_expected_consumption_for_outputs(level, low);
    }

    let c0 = layer_expected_consumption_for_outputs(level, low);
    let c1 = layer_expected_consumption_for_outputs(level, low + 1);
    c0 + frac * (c1 - c0)
}

// 有保底理论期望（近似，分层DP）
// 从顶层向下递推：
// - 先求获得 1 个 target 需要消耗多少个 target-1
// - 再把这个期望消耗作为下一层的产出需求
// - 逐层递推到 1 级，得到一级蛋期望消耗
pub fn theoretical_cost_with_pity_approx(target: usize) -> f64 {
    assert!((1..=7).contains(&target), "target 必须在 1..=7");

    let mut required_outputs = 1.0_f64;

    for level in (1..target).rev() {
        required_outputs =
            layer_expected_consumption_for_fractional_outputs(level, required_outputs);
    }

    required_outputs
}

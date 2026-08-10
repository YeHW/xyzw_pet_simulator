use super::types::TargetCostSimulationResult;

fn percentile(sorted: &[usize], q: f64) -> usize {
    let rank = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[rank]
}

pub(crate) fn summarize_target_cost_samples(
    samples: Vec<usize>,
    threads: usize,
) -> TargetCostSimulationResult {
    let mut sorted = samples.clone();
    sorted.sort_unstable();

    let trials = samples.len();
    let sum: u128 = samples.iter().map(|&x| x as u128).sum();
    let mean = sum as f64 / trials as f64;

    let variance = if trials > 1 {
        let sq_sum: f64 = samples
            .iter()
            .map(|&x| {
                let d = x as f64 - mean;
                d * d
            })
            .sum();
        sq_sum / (trials as f64 - 1.0)
    } else {
        0.0
    };

    let std_dev = variance.sqrt();
    let ci_margin = if trials > 1 {
        1.96 * std_dev / (trials as f64).sqrt()
    } else {
        0.0
    };

    let min = sorted[0];
    let max = sorted[sorted.len() - 1];

    TargetCostSimulationResult {
        trials,
        threads,
        mean,
        std_dev,
        ci95_low: mean - ci_margin,
        ci95_high: mean + ci_margin,
        min,
        max,
        p50: percentile(&sorted, 0.50),
        p90: percentile(&sorted, 0.90),
        p95: percentile(&sorted, 0.95),
        samples,
    }
}

#[cfg(test)]
mod tests {
    use super::summarize_target_cost_samples;

    #[test]
    fn preserves_trial_order_while_computing_order_statistics() {
        let result = summarize_target_cost_samples(vec![9, 1, 5], 1);

        assert_eq!(result.samples, vec![9, 1, 5]);
        assert_eq!(result.min, 1);
        assert_eq!(result.p50, 5);
        assert_eq!(result.max, 9);
    }
}

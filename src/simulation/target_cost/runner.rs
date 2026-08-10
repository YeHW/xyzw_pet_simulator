use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::{GameState, check_pity, find_merge_level, open_egg, try_merge};

use super::super::parallel::{effective_threads, splitmix64};
use super::stats::summarize_target_cost_samples;
use super::types::TargetCostSimulationResult;

fn simulate_target_cost_once_with_rng(
    target: usize,
    enable_pity: bool,
    rng: &mut impl Rng,
) -> usize {
    let mut state = GameState::new();

    loop {
        open_egg(&mut state);

        while let Some(level) = find_merge_level(&state, target) {
            try_merge(&mut state, level, rng);

            if enable_pity {
                check_pity(&mut state);
            }

            if state.pets[target] > 0 {
                return state.c1;
            }
        }
    }
}

pub fn simulate_target_cost_once(target: usize, enable_pity: bool) -> usize {
    let mut rng = rand::rng();
    simulate_target_cost_once_with_rng(target, enable_pity, &mut rng)
}

pub fn simulate_target_cost_trials(
    target: usize,
    trials: usize,
    requested_threads: Option<usize>,
    seed: Option<u64>,
    enable_pity: bool,
) -> TargetCostSimulationResult {
    assert!(trials > 0, "trials 必须大于 0");

    let threads = effective_threads(trials, requested_threads);

    let base = trials / threads;
    let rem = trials % threads;
    let mut chunks: Vec<Vec<usize>> = Vec::with_capacity(threads);

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(threads);

        for i in 0..threads {
            let count = base + usize::from(i < rem);
            let worker_seed = seed.map(|s| splitmix64(s ^ (i as u64)));
            handles.push(scope.spawn(move || {
                let mut rng = match worker_seed {
                    Some(s) => StdRng::seed_from_u64(s),
                    None => {
                        let mut trng = rand::rng();
                        StdRng::from_rng(&mut trng)
                    }
                };

                let mut local = Vec::with_capacity(count);
                for _ in 0..count {
                    local.push(simulate_target_cost_once_with_rng(
                        target,
                        enable_pity,
                        &mut rng,
                    ));
                }
                local
            }));
        }

        for handle in handles {
            chunks.push(handle.join().expect("线程执行失败"));
        }
    });

    let mut samples = Vec::with_capacity(trials);
    for mut chunk in chunks {
        samples.append(&mut chunk);
    }

    summarize_target_cost_samples(samples, threads)
}

pub fn single_target_cost_result(consumption: usize) -> TargetCostSimulationResult {
    TargetCostSimulationResult {
        trials: 1,
        threads: 1,
        mean: consumption as f64,
        std_dev: 0.0,
        ci95_low: consumption as f64,
        ci95_high: consumption as f64,
        min: consumption,
        max: consumption,
        p50: consumption,
        p90: consumption,
        p95: consumption,
        samples: vec![consumption],
    }
}

pub fn relative_error_percent(observed: f64, expected: f64) -> f64 {
    (observed - expected) / expected * 100.0
}

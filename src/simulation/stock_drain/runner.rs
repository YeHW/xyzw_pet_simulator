use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::{GameState, check_pity, try_merge};

use super::super::parallel::{effective_threads, splitmix64};
use super::types::{
    StockDrainRequest, StockDrainSimulationResult, StockDrainTrialRequest, StockDrainTrialResult,
};

fn simulate_stock_drain_once(
    stock: [usize; 6],
    enable_pity: bool,
    rng: &mut impl Rng,
) -> StockDrainTrialResult {
    let mut state = GameState::new();
    state.pets[1..=6].copy_from_slice(&stock);

    let mut pity_used = [0usize; 8];

    for level in 1..=6 {
        while state.pets[level] >= 2 {
            try_merge(&mut state, level, rng);

            if enable_pity {
                let redeemed = check_pity(&mut state);
                for pity_level in 2..=7 {
                    pity_used[pity_level] += redeemed[pity_level];
                }
            }
        }
    }

    StockDrainTrialResult {
        pets: std::array::from_fn(|index| state.pets[index + 1]),
        pity: std::array::from_fn(|index| state.pity[index + 2]),
        pity_used: std::array::from_fn(|index| pity_used[index + 2]),
    }
}

pub fn run_stock_drain_trial(request: StockDrainTrialRequest) -> StockDrainTrialResult {
    let mut rng = match request.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => {
            let mut thread_rng = rand::rng();
            StdRng::from_rng(&mut thread_rng)
        }
    };

    simulate_stock_drain_once(request.stock, request.enable_pity, &mut rng)
}

pub(super) fn simulate_stock_drain_trials(
    request: StockDrainRequest,
) -> StockDrainSimulationResult {
    let threads = effective_threads(request.trials, request.requested_threads);
    let base = request.trials / threads;
    let remainder = request.trials % threads;
    let mut chunks = Vec::with_capacity(threads);

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(threads);

        for worker_index in 0..threads {
            let count = base + usize::from(worker_index < remainder);
            let worker_seed = request
                .seed
                .map(|seed| splitmix64(seed ^ worker_index as u64));
            handles.push(scope.spawn(move || {
                let mut rng = match worker_seed {
                    Some(seed) => StdRng::seed_from_u64(seed),
                    None => {
                        let mut thread_rng = rand::rng();
                        StdRng::from_rng(&mut thread_rng)
                    }
                };
                let mut samples = Vec::with_capacity(count);

                for _ in 0..count {
                    samples.push(simulate_stock_drain_once(
                        request.stock,
                        request.enable_pity,
                        &mut rng,
                    ));
                }

                samples
            }));
        }

        for handle in handles {
            chunks.push(handle.join().expect("线程执行失败"));
        }
    });

    let mut samples = Vec::with_capacity(request.trials);
    for mut chunk in chunks {
        samples.append(&mut chunk);
    }

    StockDrainSimulationResult {
        trials: request.trials,
        threads,
        samples,
    }
}

#[cfg(test)]
mod tests {
    use crate::game::pity_threshold;

    use super::*;

    fn request(stock: [usize; 6], seed: u64, enable_pity: bool) -> StockDrainTrialRequest {
        StockDrainTrialRequest {
            stock,
            seed: Some(seed),
            enable_pity,
        }
    }

    #[test]
    fn empty_stock_stays_empty() {
        let result = run_stock_drain_trial(request([0; 6], 123, true));

        assert_eq!(result.pets, [0; 7]);
        assert_eq!(result.pity, [0; 6]);
        assert_eq!(result.pity_used, [0; 6]);
    }

    #[test]
    fn fixed_seed_is_reproducible() {
        let request = request([200, 2, 0, 0, 0, 3], 111, true);

        assert_eq!(
            run_stock_drain_trial(request),
            run_stock_drain_trial(request)
        );
    }

    #[test]
    fn drains_every_mergeable_level() {
        let result = run_stock_drain_trial(request([200, 2, 0, 0, 0, 3], 111, true));

        assert!(result.pets[..6].iter().all(|&count| count < 2));
    }

    #[test]
    fn immediately_redeems_and_counts_pity() {
        let result = (0..10_000)
            .map(|seed| run_stock_drain_trial(request([0, 0, 0, 0, 0, 31], seed, true)))
            .find(|result| result.pity_used[5] > 0)
            .expect("测试种子范围内应至少有一次 7 级保底兑换");

        assert!(result.pets[6] > 0);
        assert!(result.pity[5] < pity_threshold(7));
        assert!(result.pets[..6].iter().all(|&count| count < 2));
    }
}

use super::runner::simulate_stock_drain_trials;
use super::types::{StockDrainRequest, StockDrainSimulationResult};

pub fn run_stock_drain(request: StockDrainRequest) -> Result<StockDrainSimulationResult, String> {
    if request.trials == 0 {
        return Err("trials 必须 >= 1".to_string());
    }

    Ok(simulate_stock_drain_trials(request))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(trials: usize, threads: Option<usize>) -> StockDrainRequest {
        StockDrainRequest {
            stock: [200, 2, 0, 0, 0, 3],
            trials,
            requested_threads: threads,
            seed: Some(111),
            enable_pity: true,
        }
    }

    #[test]
    fn rejects_zero_trials() {
        assert_eq!(
            run_stock_drain(request(0, Some(1))).unwrap_err(),
            "trials 必须 >= 1"
        );
    }

    #[test]
    fn returns_one_terminal_state_per_trial() {
        let result = run_stock_drain(request(25, Some(4))).unwrap();

        assert_eq!(result.trials, 25);
        assert_eq!(result.threads, 4);
        assert_eq!(result.samples.len(), 25);
        assert!(
            result
                .samples
                .iter()
                .all(|sample| sample.pets[..6].iter().all(|&count| count < 2))
        );
    }

    #[test]
    fn fixed_seed_and_thread_count_are_reproducible() {
        assert_eq!(
            run_stock_drain(request(25, Some(4))).unwrap(),
            run_stock_drain(request(25, Some(4))).unwrap()
        );
    }
}

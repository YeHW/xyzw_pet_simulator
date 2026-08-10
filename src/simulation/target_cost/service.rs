use super::runner::{
    relative_error_percent, simulate_target_cost_once, simulate_target_cost_trials,
    single_target_cost_result,
};
use super::theory::{theoretical_cost_no_pity, theoretical_cost_with_pity_approx};
use super::types::TargetCostSimulationResult;

#[derive(Clone, Copy, Debug)]
pub enum TargetCostTheoryMode {
    Auto,
    None,
    NoPity,
    PityDp,
    All,
}

impl TargetCostTheoryMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::NoPity => "no-pity",
            Self::PityDp => "pity-dp",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TargetCostRequest {
    pub target: usize,
    pub trials: usize,
    pub requested_threads: Option<usize>,
    pub seed: Option<u64>,
    pub enable_pity: bool,
    pub theory_mode: TargetCostTheoryMode,
}

pub struct TargetCostOutcome {
    pub simulation: TargetCostSimulationResult,
    pub theory_mode: TargetCostTheoryMode,
    pub theo_no_pity: Option<f64>,
    pub theo_pity_dp: Option<f64>,
    pub relative_error_percent: Option<f64>,
}

fn resolve_theory_mode(mode: TargetCostTheoryMode, enable_pity: bool) -> TargetCostTheoryMode {
    match mode {
        TargetCostTheoryMode::Auto => {
            if enable_pity {
                TargetCostTheoryMode::PityDp
            } else {
                TargetCostTheoryMode::NoPity
            }
        }
        other => other,
    }
}

pub fn run_target_cost(request: TargetCostRequest) -> Result<TargetCostOutcome, String> {
    if !(2..=7).contains(&request.target) {
        return Err("目标等级必须在 2 到 7 之间".to_string());
    }
    if request.trials == 0 {
        return Err("trials 必须 >= 1".to_string());
    }

    let simulation_result = if request.trials == 1 {
        let consumption =
            simulate_target_cost_once(request.target, request.seed, request.enable_pity);
        single_target_cost_result(consumption)
    } else {
        simulate_target_cost_trials(
            request.target,
            request.trials,
            request.requested_threads,
            request.seed,
            request.enable_pity,
        )
    };

    let theory_mode = resolve_theory_mode(request.theory_mode, request.enable_pity);
    let show_no_pity = matches!(
        theory_mode,
        TargetCostTheoryMode::NoPity | TargetCostTheoryMode::All
    );
    let show_pity_dp = matches!(
        theory_mode,
        TargetCostTheoryMode::PityDp | TargetCostTheoryMode::All
    );

    let theo_no_pity = if show_no_pity {
        Some(theoretical_cost_no_pity(request.target))
    } else {
        None
    };

    let theo_pity_dp = if show_pity_dp {
        Some(theoretical_cost_with_pity_approx(request.target))
    } else {
        None
    };

    let relative_error = if request.enable_pity {
        theo_pity_dp.map(|v| relative_error_percent(simulation_result.mean, v))
    } else {
        theo_no_pity.map(|v| relative_error_percent(simulation_result.mean, v))
    };

    Ok(TargetCostOutcome {
        simulation: simulation_result,
        theory_mode,
        theo_no_pity,
        theo_pity_dp,
        relative_error_percent: relative_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        target: usize,
        trials: usize,
        enable_pity: bool,
        theory_mode: TargetCostTheoryMode,
    ) -> TargetCostRequest {
        TargetCostRequest {
            target,
            trials,
            requested_threads: Some(1),
            seed: Some(123),
            enable_pity,
            theory_mode,
        }
    }

    #[test]
    fn rejects_targets_outside_supported_range() {
        for target in [1, 8] {
            assert_eq!(
                run_target_cost(request(target, 1, true, TargetCostTheoryMode::None))
                    .err()
                    .as_deref(),
                Some("目标等级必须在 2 到 7 之间")
            );
        }
    }

    #[test]
    fn rejects_zero_trials() {
        assert_eq!(
            run_target_cost(request(2, 0, true, TargetCostTheoryMode::None))
                .err()
                .as_deref(),
            Some("trials 必须 >= 1")
        );
    }

    #[test]
    fn auto_mode_selects_theory_for_pity_setting() {
        let with_pity = run_target_cost(request(2, 1, true, TargetCostTheoryMode::Auto)).unwrap();
        assert_eq!(with_pity.theory_mode.as_str(), "pity-dp");
        assert!(with_pity.theo_no_pity.is_none());
        assert!(with_pity.theo_pity_dp.is_some());

        let without_pity =
            run_target_cost(request(2, 1, false, TargetCostTheoryMode::Auto)).unwrap();
        assert_eq!(without_pity.theory_mode.as_str(), "no-pity");
        assert!(without_pity.theo_no_pity.is_some());
        assert!(without_pity.theo_pity_dp.is_none());
    }
}

mod options;

use clap::Parser;
use std::io;

use crate::report::{
    HistogramOutlierConfig, HistogramOutlierMode, StockDrainSummaryJsonInput,
    TargetCostSummaryJsonInput, build_stock_drain_run_tag, build_target_cost_run_tag,
    print_histogram, print_relative_error, print_target_cost_detailed_summary,
    print_target_cost_quiet_summary, print_theory_value, resolve_output_path,
    write_stock_drain_samples_csv, write_stock_drain_summary_json, write_target_cost_samples_csv,
    write_target_cost_summary_json,
};
use crate::simulation::{
    StockDrainRequest, StockDrainSimulationResult, StockDrainTrialRequest, StockDrainTrialResult,
    TargetCostRequest, TargetCostTheoryMode as ServiceTargetCostTheoryMode, run_stock_drain,
    run_stock_drain_trial, run_target_cost,
};
use options::{
    CliCommand, CliOptions, HistOutlierMode as CliHistOutlierMode, StockDrainOptions,
    TargetCostOptions, TargetCostTheoryMode as CliTargetCostTheoryMode,
};

fn read_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
    input
}

pub fn run() {
    let command: Vec<String> = std::env::args().collect();
    let options = CliOptions::parse();

    match options.command {
        CliCommand::TargetCost(options) => execute_target_cost_command(options, command),
        CliCommand::StockDrain(options) => execute_stock_drain_command(options, command),
    }
}

fn execute_stock_drain_command(options: StockDrainOptions, command: Vec<String>) {
    let StockDrainOptions {
        stock,
        trials,
        threads,
        seed,
        disable_pity,
        csv,
        output_json,
        quiet,
    } = options;

    let enable_pity = !disable_pity;
    let result = if trials == 1 {
        let result = run_stock_drain_trial(StockDrainTrialRequest {
            stock,
            seed: Some(seed),
            enable_pity,
        });
        StockDrainSimulationResult {
            trials: 1,
            threads: 1,
            samples: vec![result],
        }
    } else {
        match run_stock_drain(StockDrainRequest {
            stock,
            trials,
            requested_threads: (threads != 0).then_some(threads),
            seed: Some(seed),
            enable_pity,
        }) {
            Ok(result) => result,
            Err(message) => {
                eprintln!("{}", message);
                std::process::exit(1);
            }
        }
    };

    print_stock_drain_result(&result, stock, enable_pity, quiet);

    let run_tag = build_stock_drain_run_tag(stock, trials, result.threads, enable_pity, Some(seed));
    let default_csv_path = format!("samples_{}.csv", run_tag);
    let default_json_path = format!("report_{}.json", run_tag);
    let csv_path = resolve_output_path(csv, &default_csv_path);
    let json_path = resolve_output_path(output_json, &default_json_path);

    if let Some(path) = csv_path.as_deref() {
        write_stock_drain_samples_csv(path, &result.samples, enable_pity)
            .expect("写入 stock-drain CSV 失败");
        println!("CSV 已导出到: {}", path);
    }

    if let Some(path) = json_path.as_deref() {
        write_stock_drain_summary_json(StockDrainSummaryJsonInput {
            path,
            command: &command,
            stock,
            enable_pity,
            seed: Some(seed),
            output_csv_path: csv_path.as_deref(),
            output_json_path: Some(path),
            simulation: &result,
        })
        .expect("写入 stock-drain JSON 失败");
        println!("JSON 已导出到: {}", path);
    }
}

fn print_stock_drain_result(
    result: &StockDrainSimulationResult,
    stock: [usize; 6],
    enable_pity: bool,
    quiet: bool,
) {
    println!("起始资源量 (1-6级): {:?}", stock);

    if quiet {
        print_stock_drain_means(result, enable_pity);
        return;
    }

    if result.trials == 1 {
        print_stock_drain_trial(&result.samples[0], enable_pity);
        return;
    }

    println!("库存耗尽模拟摘要");
    println!("模拟次数: {}", result.trials);
    println!("线程数: {}", result.threads);
    let pets = summarize_columns::<7>(&result.samples, |sample| sample.pets);
    print_pet_summary(&pets);
    print_stock_drain_pity_means(result, enable_pity);
}

fn print_stock_drain_means(result: &StockDrainSimulationResult, enable_pity: bool) {
    println!(
        "最终宠物均值 (1-7级): {:.3?}",
        column_means::<7>(&result.samples, |sample| sample.pets)
    );
    print_stock_drain_pity_means(result, enable_pity);
}

fn print_stock_drain_pity_means(result: &StockDrainSimulationResult, enable_pity: bool) {
    if enable_pity {
        println!(
            "剩余保底进度均值 (2-7级): {:.3?}",
            column_means::<6>(&result.samples, |sample| sample.pity)
        );
        println!(
            "保底使用次数均值 (2-7级): {:.3?}",
            column_means::<6>(&result.samples, |sample| sample.pity_used)
        );
    }
}

struct ColumnSummary<const N: usize> {
    mean: [f64; N],
    p5: [usize; N],
    p10: [usize; N],
    p20: [usize; N],
    p30: [usize; N],
    p40: [usize; N],
    p50: [usize; N],
    p60: [usize; N],
    p70: [usize; N],
    p80: [usize; N],
    p90: [usize; N],
    p95: [usize; N],
}

fn summarize_columns<const N: usize>(
    samples: &[StockDrainTrialResult],
    select: impl Fn(&StockDrainTrialResult) -> [usize; N],
) -> ColumnSummary<N> {
    let mut totals = [0.0; N];
    let mut columns: [Vec<usize>; N] = std::array::from_fn(|_| Vec::with_capacity(samples.len()));
    for sample in samples {
        for (column, (total, value)) in totals.iter_mut().zip(select(sample)).enumerate() {
            *total += value as f64;
            columns[column].push(value);
        }
    }

    for column in &mut columns {
        column.sort_unstable();
    }

    ColumnSummary {
        mean: totals.map(|total| total / samples.len() as f64),
        p5: std::array::from_fn(|column| percentile(&columns[column], 0.05)),
        p10: std::array::from_fn(|column| percentile(&columns[column], 0.10)),
        p20: std::array::from_fn(|column| percentile(&columns[column], 0.20)),
        p30: std::array::from_fn(|column| percentile(&columns[column], 0.30)),
        p40: std::array::from_fn(|column| percentile(&columns[column], 0.40)),
        p50: std::array::from_fn(|column| percentile(&columns[column], 0.50)),
        p60: std::array::from_fn(|column| percentile(&columns[column], 0.60)),
        p70: std::array::from_fn(|column| percentile(&columns[column], 0.70)),
        p80: std::array::from_fn(|column| percentile(&columns[column], 0.80)),
        p90: std::array::from_fn(|column| percentile(&columns[column], 0.90)),
        p95: std::array::from_fn(|column| percentile(&columns[column], 0.95)),
    }
}

fn column_means<const N: usize>(
    samples: &[StockDrainTrialResult],
    select: impl Fn(&StockDrainTrialResult) -> [usize; N],
) -> [f64; N] {
    let mut totals = [0.0; N];
    for sample in samples {
        for (total, value) in totals.iter_mut().zip(select(sample)) {
            *total += value as f64;
        }
    }

    totals.map(|total| total / samples.len() as f64)
}

fn percentile(sorted: &[usize], quantile: f64) -> usize {
    let rank = ((sorted.len() - 1) as f64 * quantile).round() as usize;
    sorted[rank]
}

fn print_pet_summary(summary: &ColumnSummary<7>) {
    println!("最终宠物均值 (1-7级): {:.3?}", summary.mean);
    println!("最终宠物 P5 (1-7级): {:?}", summary.p5);
    println!("最终宠物 P10 (1-7级): {:?}", summary.p10);
    println!("最终宠物 P20 (1-7级): {:?}", summary.p20);
    println!("最终宠物 P30 (1-7级): {:?}", summary.p30);
    println!("最终宠物 P40 (1-7级): {:?}", summary.p40);
    println!("最终宠物 P50 (1-7级): {:?}", summary.p50);
    println!("最终宠物 P60 (1-7级): {:?}", summary.p60);
    println!("最终宠物 P70 (1-7级): {:?}", summary.p70);
    println!("最终宠物 P80 (1-7级): {:?}", summary.p80);
    println!("最终宠物 P90 (1-7级): {:?}", summary.p90);
    println!("最终宠物 P95 (1-7级): {:?}", summary.p95);
}

#[cfg(test)]
mod tests {
    use super::{column_means, summarize_columns};
    use crate::simulation::StockDrainTrialResult;

    #[test]
    fn computes_stock_drain_column_summaries() {
        let samples: Vec<_> = (0..10)
            .map(|value| StockDrainTrialResult {
                pets: [value; 7],
                pity: [value; 6],
                pity_used: [0; 6],
            })
            .collect();

        let summary = summarize_columns::<7>(&samples, |sample| sample.pets);

        assert_eq!(summary.mean, [4.5; 7]);
        assert_eq!(summary.p5, [0; 7]);
        assert_eq!(summary.p10, [1; 7]);
        assert_eq!(summary.p20, [2; 7]);
        assert_eq!(summary.p30, [3; 7]);
        assert_eq!(summary.p40, [4; 7]);
        assert_eq!(summary.p50, [5; 7]);
        assert_eq!(summary.p60, [5; 7]);
        assert_eq!(summary.p70, [6; 7]);
        assert_eq!(summary.p80, [7; 7]);
        assert_eq!(summary.p90, [8; 7]);
        assert_eq!(summary.p95, [9; 7]);
        assert_eq!(column_means::<6>(&samples, |sample| sample.pity), [4.5; 6]);
    }
}

fn print_stock_drain_trial(result: &StockDrainTrialResult, enable_pity: bool) {
    println!("最终宠物 (1-7级): {:?}", result.pets);
    if enable_pity {
        println!("剩余保底进度 (2-7级): {:?}", result.pity);
        println!("保底使用次数 (2-7级): {:?}", result.pity_used);
    }
}

fn execute_target_cost_command(options: TargetCostOptions, command: Vec<String>) {
    let pity_enabled = !options.disable_pity;

    let target = options.target;
    let trials = options.trials;
    let requested_threads = if options.threads == 0 {
        None
    } else {
        Some(options.threads)
    };

    let theory_mode = match options.theory_mode {
        None => ServiceTargetCostTheoryMode::Auto,
        Some(CliTargetCostTheoryMode::None) => ServiceTargetCostTheoryMode::None,
        Some(CliTargetCostTheoryMode::NoPity) => ServiceTargetCostTheoryMode::NoPity,
        Some(CliTargetCostTheoryMode::PityDp) => ServiceTargetCostTheoryMode::PityDp,
        Some(CliTargetCostTheoryMode::All) => ServiceTargetCostTheoryMode::All,
    };

    let target_cost_outcome = match run_target_cost(TargetCostRequest {
        target,
        trials,
        requested_threads,
        seed: Some(options.seed),
        enable_pity: pity_enabled,
        theory_mode,
    }) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    let target_cost_run_tag = build_target_cost_run_tag(
        target,
        trials,
        target_cost_outcome.simulation.threads,
        pity_enabled,
        Some(options.seed),
    );
    let default_csv_path = format!("samples_{}.csv", target_cost_run_tag);
    let default_json_path = format!("report_{}.json", target_cost_run_tag);

    let quiet = options.quiet;

    if !quiet {
        print_target_cost_detailed_summary(
            target,
            options.seed,
            pity_enabled,
            &target_cost_outcome.simulation,
        );
        if let Some(v) = target_cost_outcome.theo_no_pity {
            print_theory_value("理论值(无保底, 精确)", v, true);
        }
        if let Some(v) = target_cost_outcome.theo_pity_dp {
            print_theory_value("理论值(有保底, 近似DP)", v, false);
        }
        if let Some(err) = target_cost_outcome.relative_error_percent {
            if pity_enabled {
                print_relative_error("与有保底近似理论的相对误差", err);
            } else {
                print_relative_error("与无保底精确理论的相对误差", err);
            }
        }
    }

    if !quiet {
        let outlier_mode = match options.hist_outlier_mode {
            CliHistOutlierMode::None => HistogramOutlierMode::None,
            CliHistOutlierMode::Iqr => HistogramOutlierMode::Iqr,
            CliHistOutlierMode::Mad => HistogramOutlierMode::Mad,
            CliHistOutlierMode::Quantile => HistogramOutlierMode::Quantile,
            CliHistOutlierMode::Winsor => HistogramOutlierMode::Winsor,
        };
        print_histogram(
            &target_cost_outcome.simulation.samples,
            options.bins,
            HistogramOutlierConfig {
                mode: outlier_mode,
                iqr_k: options.hist_iqr_k,
                mad_threshold: options.hist_mad_threshold,
                quantile_alpha: options.hist_quantile_alpha,
            },
        );
    }

    if quiet {
        print_target_cost_quiet_summary(
            target,
            pity_enabled,
            &target_cost_outcome.simulation,
            target_cost_outcome.relative_error_percent,
        );
    }

    let csv_path = if let Some(path) = resolve_output_path(options.csv.clone(), &default_csv_path) {
        Some(path)
    } else if options.no_interactive {
        None
    } else {
        let export = read_line("是否导出样本到 CSV? (y/n):");
        if export.trim().eq_ignore_ascii_case("y") {
            let prompt = format!("请输入 CSV 文件路径 (默认 {}):", default_csv_path);
            let path_input = read_line(&prompt);
            if path_input.trim().is_empty() {
                Some(default_csv_path.clone())
            } else {
                Some(path_input.trim().to_string())
            }
        } else {
            None
        }
    };

    if let Some(path) = csv_path.as_deref() {
        write_target_cost_samples_csv(path, &target_cost_outcome.simulation.samples)
            .expect("写入 CSV 失败");
        println!("CSV 已导出到: {}", path);
    }

    let output_json_path = resolve_output_path(options.output_json.clone(), &default_json_path);
    if let Some(path) = output_json_path.as_deref() {
        write_target_cost_summary_json(TargetCostSummaryJsonInput {
            path,
            command: &command,
            target,
            enable_pity: pity_enabled,
            theory_mode: target_cost_outcome.theory_mode.as_str(),
            seed: Some(options.seed),
            output_csv_path: csv_path.as_deref(),
            output_json_path: Some(path),
            simulation: &target_cost_outcome.simulation,
            theo_no_pity: target_cost_outcome.theo_no_pity,
            theo_pity_dp: target_cost_outcome.theo_pity_dp,
            relative_error_percent: target_cost_outcome.relative_error_percent,
        })
        .expect("写入 JSON 失败");
        println!("JSON 已导出到: {}", path);
    }
}

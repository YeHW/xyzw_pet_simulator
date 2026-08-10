use serde::Serialize;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::simulation::{
    StockDrainSimulationResult, StockDrainTrialResult, TargetCostSimulationResult,
};

#[derive(Serialize)]
struct TargetCostSummaryConfig {
    target: usize,
    enable_pity: bool,
    theory_mode: String,
    seed: Option<u64>,
    command: Vec<String>,
    output_csv_path: Option<String>,
    output_json_path: Option<String>,
}

#[derive(Serialize)]
struct TargetCostSummaryResult {
    trials: usize,
    threads: usize,
    mean: f64,
    std_dev: f64,
    ci95_low: f64,
    ci95_high: f64,
    min: usize,
    p50: usize,
    p90: usize,
    p95: usize,
    max: usize,
}

#[derive(Serialize)]
struct TargetCostSummaryTheory {
    no_pity_exact: Option<f64>,
    pity_dp_approx: Option<f64>,
    relative_error_percent: Option<f64>,
}

#[derive(Serialize)]
struct TargetCostSummaryJson {
    config: TargetCostSummaryConfig,
    result: TargetCostSummaryResult,
    theory: TargetCostSummaryTheory,
}

pub struct TargetCostSummaryJsonInput<'a> {
    pub path: &'a str,
    pub command: &'a [String],
    pub target: usize,
    pub enable_pity: bool,
    pub theory_mode: &'a str,
    pub seed: Option<u64>,
    pub output_csv_path: Option<&'a str>,
    pub output_json_path: Option<&'a str>,
    pub simulation: &'a TargetCostSimulationResult,
    pub theo_no_pity: Option<f64>,
    pub theo_pity_dp: Option<f64>,
    pub relative_error_percent: Option<f64>,
}

pub fn write_target_cost_samples_csv<P: AsRef<Path>>(path: P, samples: &[usize]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "trial,eggs")?;
    for (idx, value) in samples.iter().enumerate() {
        writeln!(writer, "{},{}", idx + 1, value)?;
    }

    writer.flush()
}

pub fn write_target_cost_summary_json(input: TargetCostSummaryJsonInput<'_>) -> io::Result<()> {
    let summary = TargetCostSummaryJson {
        config: TargetCostSummaryConfig {
            target: input.target,
            enable_pity: input.enable_pity,
            theory_mode: input.theory_mode.to_string(),
            seed: input.seed,
            command: input.command.to_vec(),
            output_csv_path: input.output_csv_path.map(|s| s.to_string()),
            output_json_path: input.output_json_path.map(|s| s.to_string()),
        },
        result: TargetCostSummaryResult {
            trials: input.simulation.trials,
            threads: input.simulation.threads,
            mean: input.simulation.mean,
            std_dev: input.simulation.std_dev,
            ci95_low: input.simulation.ci95_low,
            ci95_high: input.simulation.ci95_high,
            min: input.simulation.min,
            p50: input.simulation.p50,
            p90: input.simulation.p90,
            p95: input.simulation.p95,
            max: input.simulation.max,
        },
        theory: TargetCostSummaryTheory {
            no_pity_exact: input.theo_no_pity,
            pity_dp_approx: input.theo_pity_dp,
            relative_error_percent: input.relative_error_percent,
        },
    };

    let mut file = File::create(input.path)?;
    serde_json::to_writer_pretty(&mut file, &summary).map_err(io::Error::other)
}

#[derive(Serialize)]
struct StockDrainSummaryConfig {
    stock: [usize; 6],
    enable_pity: bool,
    seed: Option<u64>,
    command: Vec<String>,
    output_csv_path: Option<String>,
    output_json_path: Option<String>,
}

#[derive(Serialize)]
struct StockDrainColumnSummary {
    level: usize,
    mean: f64,
    min: usize,
    p5: usize,
    p10: usize,
    p20: usize,
    p30: usize,
    p40: usize,
    p50: usize,
    p60: usize,
    p70: usize,
    p80: usize,
    p90: usize,
    p95: usize,
    max: usize,
}

#[derive(Serialize)]
struct StockDrainPitySummary {
    progress_by_level: Vec<StockDrainColumnSummary>,
    used_by_level: Vec<StockDrainColumnSummary>,
}

#[derive(Serialize)]
struct StockDrainSummaryResult {
    trials: usize,
    threads: usize,
    pets_by_level: Vec<StockDrainColumnSummary>,
    pity: Option<StockDrainPitySummary>,
}

#[derive(Serialize)]
struct StockDrainSummaryJson {
    config: StockDrainSummaryConfig,
    result: StockDrainSummaryResult,
}

pub struct StockDrainSummaryJsonInput<'a> {
    pub path: &'a str,
    pub command: &'a [String],
    pub stock: [usize; 6],
    pub enable_pity: bool,
    pub seed: Option<u64>,
    pub output_csv_path: Option<&'a str>,
    pub output_json_path: Option<&'a str>,
    pub simulation: &'a StockDrainSimulationResult,
}

pub fn write_stock_drain_samples_csv<P: AsRef<Path>>(
    path: P,
    samples: &[StockDrainTrialResult],
    enable_pity: bool,
) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    write!(
        writer,
        "trial,level1,level2,level3,level4,level5,level6,level7"
    )?;
    if enable_pity {
        write!(
            writer,
            ",pity2,pity3,pity4,pity5,pity6,pity7,pity_used2,pity_used3,pity_used4,pity_used5,pity_used6,pity_used7"
        )?;
    }
    writeln!(writer)?;

    for (index, sample) in samples.iter().enumerate() {
        write!(writer, "{}", index + 1)?;
        write_values(&mut writer, sample.pets)?;
        if enable_pity {
            write_values(&mut writer, sample.pity)?;
            write_values(&mut writer, sample.pity_used)?;
        }
        writeln!(writer)?;
    }

    writer.flush()
}

fn write_values<const N: usize>(writer: &mut impl Write, values: [usize; N]) -> io::Result<()> {
    for value in values {
        write!(writer, ",{}", value)?;
    }
    Ok(())
}

pub fn write_stock_drain_summary_json(input: StockDrainSummaryJsonInput<'_>) -> io::Result<()> {
    let samples = &input.simulation.samples;
    let pity = input.enable_pity.then(|| StockDrainPitySummary {
        progress_by_level: summarize_stock_drain_columns::<6>(samples, 2, |sample| sample.pity),
        used_by_level: summarize_stock_drain_columns::<6>(samples, 2, |sample| sample.pity_used),
    });
    let summary = StockDrainSummaryJson {
        config: StockDrainSummaryConfig {
            stock: input.stock,
            enable_pity: input.enable_pity,
            seed: input.seed,
            command: input.command.to_vec(),
            output_csv_path: input.output_csv_path.map(str::to_string),
            output_json_path: input.output_json_path.map(str::to_string),
        },
        result: StockDrainSummaryResult {
            trials: input.simulation.trials,
            threads: input.simulation.threads,
            pets_by_level: summarize_stock_drain_columns::<7>(samples, 1, |sample| sample.pets),
            pity,
        },
    };

    let mut file = File::create(input.path)?;
    serde_json::to_writer_pretty(&mut file, &summary).map_err(io::Error::other)
}

fn summarize_stock_drain_columns<const N: usize>(
    samples: &[StockDrainTrialResult],
    first_level: usize,
    select: impl Fn(&StockDrainTrialResult) -> [usize; N],
) -> Vec<StockDrainColumnSummary> {
    (0..N)
        .map(|column| {
            let mut values: Vec<_> = samples
                .iter()
                .map(|sample| select(sample)[column])
                .collect();
            values.sort_unstable();
            let mean = values.iter().map(|&value| value as u128).sum::<u128>() as f64
                / values.len() as f64;

            StockDrainColumnSummary {
                level: first_level + column,
                mean,
                min: values[0],
                p5: percentile(&values, 0.05),
                p10: percentile(&values, 0.10),
                p20: percentile(&values, 0.20),
                p30: percentile(&values, 0.30),
                p40: percentile(&values, 0.40),
                p50: percentile(&values, 0.50),
                p60: percentile(&values, 0.60),
                p70: percentile(&values, 0.70),
                p80: percentile(&values, 0.80),
                p90: percentile(&values, 0.90),
                p95: percentile(&values, 0.95),
                max: values[values.len() - 1],
            }
        })
        .collect()
}

fn percentile(sorted: &[usize], quantile: f64) -> usize {
    let rank = ((sorted.len() - 1) as f64 * quantile).round() as usize;
    sorted[rank]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples() -> Vec<StockDrainTrialResult> {
        vec![
            StockDrainTrialResult {
                pets: [0, 1, 0, 1, 0, 1, 0],
                pity: [1, 2, 3, 4, 5, 6],
                pity_used: [0, 1, 0, 1, 0, 1],
            },
            StockDrainTrialResult {
                pets: [1, 0, 0, 1, 0, 1, 2],
                pity: [3, 4, 5, 6, 7, 8],
                pity_used: [2, 1, 2, 1, 2, 1],
            },
        ]
    }

    fn test_path(extension: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "pet_merge_stock_drain_export_{}.{}",
            std::process::id(),
            extension
        ))
    }

    #[test]
    fn writes_stock_drain_csv_with_pity_columns() {
        let path = test_path("csv");
        write_stock_drain_samples_csv(&path, &samples(), true).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(path).unwrap();

        let mut lines = content.lines();
        assert_eq!(lines.next().unwrap().split(',').count(), 20);
        assert_eq!(lines.next().unwrap().split(',').count(), 20);
        assert_eq!(lines.count(), 1);
    }

    #[test]
    fn writes_stock_drain_json_column_summaries() {
        let path = test_path("json");
        let simulation = StockDrainSimulationResult {
            trials: 2,
            threads: 1,
            samples: samples(),
        };
        write_stock_drain_summary_json(StockDrainSummaryJsonInput {
            path: path.to_str().unwrap(),
            command: &["xyzw-petsim".to_string(), "stock-drain".to_string()],
            stock: [2, 0, 0, 0, 0, 0],
            enable_pity: true,
            seed: Some(123),
            output_csv_path: Some("samples.csv"),
            output_json_path: Some(path.to_str().unwrap()),
            simulation: &simulation,
        })
        .unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(value["result"]["trials"], 2);
        assert_eq!(value["result"]["pets_by_level"][0]["mean"], 0.5);
        assert_eq!(value["result"]["pity"]["progress_by_level"][0]["mean"], 2.0);
        for path in [
            &value["result"]["pets_by_level"][0],
            &value["result"]["pity"]["progress_by_level"][0],
            &value["result"]["pity"]["used_by_level"][0],
        ] {
            for percentile in [
                "p5", "p10", "p20", "p30", "p40", "p50", "p60", "p70", "p80", "p90", "p95",
            ] {
                assert!(path.get(percentile).is_some(), "缺少字段 {percentile}");
            }
        }
    }
}

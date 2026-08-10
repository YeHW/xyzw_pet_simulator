use crate::simulation::TargetCostSimulationResult;

#[derive(Clone, Copy, Debug)]
pub enum HistogramOutlierMode {
    None,
    Iqr,
    Mad,
    Quantile,
    Winsor,
}

#[derive(Clone, Copy, Debug)]
pub struct HistogramOutlierConfig {
    pub mode: HistogramOutlierMode,
    pub iqr_k: f64,
    pub mad_threshold: f64,
    pub quantile_alpha: f64,
}

impl Default for HistogramOutlierConfig {
    fn default() -> Self {
        Self {
            mode: HistogramOutlierMode::None,
            iqr_k: 1.5,
            mad_threshold: 3.5,
            quantile_alpha: 0.01,
        }
    }
}

fn percentile(sorted: &[usize], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len();
    if n == 1 {
        return sorted[0] as f64;
    }

    let rank = p.clamp(0.0, 1.0) * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;

    if lo == hi {
        sorted[lo] as f64
    } else {
        let t = rank - lo as f64;
        sorted[lo] as f64 * (1.0 - t) + sorted[hi] as f64 * t
    }
}

fn median_usize(sorted: &[usize]) -> f64 {
    percentile(sorted, 0.5)
}

pub fn print_histogram(samples: &[usize], bins: usize, outlier: HistogramOutlierConfig) {
    if samples.is_empty() {
        return;
    }

    let bins = bins.max(1);
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();

    let mut processed = sorted.clone();
    let mut removed_low = 0usize;
    let mut removed_high = 0usize;
    let mut capped_low = 0usize;
    let mut capped_high = 0usize;
    let mut rule_desc = String::new();

    match outlier.mode {
        HistogramOutlierMode::None => {}
        HistogramOutlierMode::Iqr => {
            let q1 = percentile(&sorted, 0.25);
            let q3 = percentile(&sorted, 0.75);
            let iqr = (q3 - q1).max(0.0);
            let low = q1 - outlier.iqr_k * iqr;
            let high = q3 + outlier.iqr_k * iqr;
            processed = sorted
                .iter()
                .copied()
                .filter(|&x| {
                    let xf = x as f64;
                    if xf < low {
                        removed_low += 1;
                        return false;
                    }
                    if xf > high {
                        removed_high += 1;
                        return false;
                    }
                    true
                })
                .collect();
            rule_desc = format!(
                "IQR: 保留 [{:.2}, {:.2}] (k={:.2})",
                low, high, outlier.iqr_k
            );
        }
        HistogramOutlierMode::Mad => {
            let med = median_usize(&sorted);
            let abs_dev: Vec<usize> = sorted
                .iter()
                .map(|&x| (x as f64 - med).abs().round() as usize)
                .collect();
            let mut abs_sorted = abs_dev;
            abs_sorted.sort_unstable();
            let mad = median_usize(&abs_sorted);
            if mad > 0.0 {
                processed = sorted
                    .iter()
                    .copied()
                    .filter(|&x| {
                        let score = 0.6745 * ((x as f64) - med) / mad;
                        if score < -outlier.mad_threshold {
                            removed_low += 1;
                            return false;
                        }
                        if score > outlier.mad_threshold {
                            removed_high += 1;
                            return false;
                        }
                        true
                    })
                    .collect();
                rule_desc = format!(
                    "MAD: |M| <= {:.2}, M=0.6745*(x-med)/MAD",
                    outlier.mad_threshold
                );
            } else {
                rule_desc = "MAD=0，无法按 MAD 过滤，已保留全部样本".to_string();
            }
        }
        HistogramOutlierMode::Quantile => {
            let low = percentile(&sorted, outlier.quantile_alpha);
            let high = percentile(&sorted, 1.0 - outlier.quantile_alpha);
            processed = sorted
                .iter()
                .copied()
                .filter(|&x| {
                    let xf = x as f64;
                    if xf < low {
                        removed_low += 1;
                        return false;
                    }
                    if xf > high {
                        removed_high += 1;
                        return false;
                    }
                    true
                })
                .collect();
            rule_desc = format!(
                "分位数截断: 保留 [{:.2}, {:.2}] (alpha={:.4})",
                low, high, outlier.quantile_alpha
            );
        }
        HistogramOutlierMode::Winsor => {
            let low = percentile(&sorted, outlier.quantile_alpha).round() as usize;
            let high = percentile(&sorted, 1.0 - outlier.quantile_alpha).round() as usize;
            processed = sorted
                .iter()
                .copied()
                .map(|x| {
                    if x < low {
                        capped_low += 1;
                        low
                    } else if x > high {
                        capped_high += 1;
                        high
                    } else {
                        x
                    }
                })
                .collect();
            processed.sort_unstable();
            rule_desc = format!(
                "Winsorize: 截帽到 [{}, {}] (alpha={:.4})",
                low, high, outlier.quantile_alpha
            );
        }
    }

    if processed.is_empty() {
        processed = sorted.clone();
        rule_desc = "过滤后样本为空，已回退为全部样本".to_string();
        removed_low = 0;
        removed_high = 0;
        capped_low = 0;
        capped_high = 0;
    }

    let min = processed[0];
    let max = processed[processed.len() - 1];

    if min == max {
        println!("\n直方图:");
        if !matches!(outlier.mode, HistogramOutlierMode::None) {
            println!("异常值处理: {}", rule_desc);
            println!(
                "样本变化: n={} -> n={}, 移除低端={}, 移除高端={}, 截帽低端={}, 截帽高端={}",
                samples.len(),
                processed.len(),
                removed_low,
                removed_high,
                capped_low,
                capped_high
            );
        }
        println!("{} | {} ({})", min, "#".repeat(40), processed.len());
        return;
    }

    let width = (max - min + 1) as f64 / bins as f64;
    let mut counts = vec![0usize; bins];

    for &x in &processed {
        let mut idx = ((x - min) as f64 / width).floor() as usize;
        if idx >= bins {
            idx = bins - 1;
        }
        counts[idx] += 1;
    }

    let max_count = *counts.iter().max().unwrap_or(&1);
    println!("\n直方图 ({} bins):", bins);
    if !matches!(outlier.mode, HistogramOutlierMode::None) {
        println!("异常值处理: {}", rule_desc);
        println!(
            "样本变化: n={} -> n={}, 移除低端={}, 移除高端={}, 截帽低端={}, 截帽高端={}",
            samples.len(),
            processed.len(),
            removed_low,
            removed_high,
            capped_low,
            capped_high
        );
    }

    for (i, &count) in counts.iter().enumerate() {
        let start = min as f64 + i as f64 * width;
        let end = min as f64 + (i + 1) as f64 * width;
        let bar_len = if max_count == 0 {
            0
        } else {
            count * 40 / max_count
        };
        println!(
            "[{:<6.0}, {:<6.0}) | {:<40} ({})",
            start,
            end,
            "#".repeat(bar_len),
            count
        );
    }
}

pub fn print_target_cost_detailed_summary(
    target: usize,
    seed: u64,
    pity_enabled: bool,
    simulation: &TargetCostSimulationResult,
) {
    println!("目标等级: {}", target);
    println!("模拟次数: {}", simulation.trials);
    println!("线程数: {}", simulation.threads);
    println!("保底兑换: {}", if pity_enabled { "开启" } else { "关闭" });
    println!("随机种子: {}", seed);
    println!("平均消耗: {:.2}", simulation.mean);
    println!("标准差: {:.2}", simulation.std_dev);
    println!(
        "95% 置信区间: [{:.2}, {:.2}]",
        simulation.ci95_low, simulation.ci95_high
    );
    println!("最小消耗: {}", simulation.min);
    println!("P50 消耗: {}", simulation.p50);
    println!("P90 消耗: {}", simulation.p90);
    println!("P95 消耗: {}", simulation.p95);
    println!("最大消耗: {}", simulation.max);
}

pub fn print_target_cost_quiet_summary(
    target: usize,
    pity_enabled: bool,
    simulation: &TargetCostSimulationResult,
    relative_error: Option<f64>,
) {
    println!("目标等级: {}", target);
    println!("模拟次数: {}", simulation.trials);
    println!("保底兑换: {}", if pity_enabled { "开启" } else { "关闭" });
    println!("平均消耗: {:.2}", simulation.mean);
    println!("P50 消耗: {}", simulation.p50);
    println!("P95 消耗: {}", simulation.p95);
    println!("最小消耗: {}", simulation.min);
    println!("最大消耗: {}", simulation.max);
    if let Some(err) = relative_error {
        println!("理论相对误差: {:+.2}%", err);
    }
}

pub fn print_theory_value(label: &str, value: f64, leading_newline: bool) {
    if leading_newline {
        println!("\n{}: {:.2}", label, value);
    } else {
        println!("{}: {:.2}", label, value);
    }
}

pub fn print_relative_error(label: &str, value: f64) {
    println!("{}: {:+.2}%", label, value);
}

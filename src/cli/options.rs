use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum TargetCostTheoryMode {
    None,
    NoPity,
    PityDp,
    All,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum HistOutlierMode {
    None,
    Iqr,
    Mad,
    Quantile,
    Winsor,
}

#[derive(Debug, Parser)]
#[command(name = "xyzw-petsim")]
#[command(about = "宠物合成蒙特卡洛模拟器")]
#[command(
    after_help = "合成概率 p(n): p1=0.95, p2=0.80, p3=0.65, p4=0.45, p5=0.20, p6=0.05\n保底阈值 r(n): r(n)=5*(n-1), n=2..7"
)]
pub(crate) struct CliOptions {
    #[command(subcommand)]
    pub(crate) command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CliCommand {
    #[command(about = "估计获得目标等级宠物所需的一级蛋消耗")]
    TargetCost(TargetCostOptions),
    #[command(about = "从初始库存开始合成，直到无法继续")]
    StockDrain(StockDrainOptions),
}

#[derive(Debug, Args)]
pub(crate) struct TargetCostOptions {
    #[arg(short = 'T', long, default_value_t = 6, help = "目标宠物等级 (2-7)")]
    pub(crate) target: usize,
    #[arg(
        short = 'N',
        long,
        default_value_t = 10_000,
        value_parser = parse_trials,
        help = "蒙特卡洛模拟次数 (>=1)"
    )]
    pub(crate) trials: usize,
    #[arg(short = 't', long, default_value_t = 1, help = "线程数 (0 表示自动)")]
    pub(crate) threads: usize,
    #[arg(
        short = 'S',
        long,
        default_value_t = 123,
        help = "随机种子 (可复现实验)"
    )]
    pub(crate) seed: u64,
    #[arg(
        short = 'C',
        long,
        num_args = 0..=1,
        default_missing_value = "__AUTO__",
        help = "导出样本 CSV；不带路径时使用默认命名"
    )]
    pub(crate) csv: Option<Option<String>>,
    #[arg(
        short = 'J',
        long = "output-json",
        num_args = 0..=1,
        default_missing_value = "__AUTO__",
        help = "导出摘要 JSON；不带路径时使用默认命名"
    )]
    pub(crate) output_json: Option<Option<String>>,
    #[arg(short = 'B', long, default_value_t = 12, help = "直方图分桶数量")]
    pub(crate) bins: usize,
    #[arg(
        long = "hist-outlier-mode",
        value_enum,
        default_value_t = HistOutlierMode::None,
        help = "直方图异常值处理模式: none/iqr/mad/quantile/winsor"
    )]
    pub(crate) hist_outlier_mode: HistOutlierMode,
    #[arg(
        long = "hist-iqr-k",
        default_value_t = 1.5,
        value_parser = parse_positive_f64,
        help = "IQR 模式参数 k，阈值为 [Q1-k*IQR, Q3+k*IQR]"
    )]
    pub(crate) hist_iqr_k: f64,
    #[arg(
        long = "hist-mad-threshold",
        default_value_t = 3.5,
        value_parser = parse_positive_f64,
        help = "MAD 模式阈值，默认 3.5"
    )]
    pub(crate) hist_mad_threshold: f64,
    #[arg(
        long = "hist-quantile-alpha",
        default_value_t = 0.01,
        value_parser = parse_quantile_alpha,
        help = "分位数模式 alpha，保留 [q(alpha), q(1-alpha)]"
    )]
    pub(crate) hist_quantile_alpha: f64,
    #[arg(short = 'D', long, help = "关闭保底兑换（默认开启）")]
    pub(crate) disable_pity: bool,
    #[arg(short = 'M', long, value_enum, help = "理论值输出模式")]
    pub(crate) theory_mode: Option<TargetCostTheoryMode>,
    #[arg(
        short = 'I',
        long = "no-interactive",
        help = "非交互模式：缺失关键参数时报错，不询问导出"
    )]
    pub(crate) no_interactive: bool,
    #[arg(short = 'q', long, help = "精简输出，仅打印摘要")]
    pub(crate) quiet: bool,
}

#[derive(Debug, Args)]
pub(crate) struct StockDrainOptions {
    #[arg(
        long,
        required = true,
        value_parser = parse_stock,
        value_name = "L1,L2,L3,L4,L5,L6",
        help = "初始 1-6 级宠物库存，例如 200,2,0,0,0,3"
    )]
    pub(crate) stock: [usize; 6],
    #[arg(
        short = 'N',
        long,
        default_value_t = 10_000,
        value_parser = parse_trials,
        help = "蒙特卡洛模拟次数 (>=1)"
    )]
    pub(crate) trials: usize,
    #[arg(short = 't', long, default_value_t = 1, help = "线程数 (0 表示自动)")]
    pub(crate) threads: usize,
    #[arg(
        short = 'S',
        long,
        default_value_t = 123,
        help = "随机种子 (可复现实验)"
    )]
    pub(crate) seed: u64,
    #[arg(short = 'D', long, help = "关闭保底兑换（默认开启）")]
    pub(crate) disable_pity: bool,
    #[arg(
        short = 'C',
        long,
        num_args = 0..=1,
        default_missing_value = "__AUTO__",
        help = "导出逐次状态矩阵 CSV；不带路径时使用默认命名"
    )]
    pub(crate) csv: Option<Option<String>>,
    #[arg(
        short = 'J',
        long = "output-json",
        num_args = 0..=1,
        default_missing_value = "__AUTO__",
        help = "导出汇总 JSON；不带路径时使用默认命名"
    )]
    pub(crate) output_json: Option<Option<String>>,
    #[arg(short = 'q', long, help = "精简输出，仅打印摘要")]
    pub(crate) quiet: bool,
}

fn parse_trials(s: &str) -> Result<usize, String> {
    let n = s
        .parse::<usize>()
        .map_err(|_| "trials 必须是正整数".to_string())?;
    if n == 0 {
        return Err("trials 必须 >= 1".to_string());
    }
    Ok(n)
}

fn parse_positive_f64(s: &str) -> Result<f64, String> {
    let v = s.parse::<f64>().map_err(|_| "参数必须是数字".to_string())?;
    if !v.is_finite() || v <= 0.0 {
        return Err("参数必须 > 0".to_string());
    }
    Ok(v)
}

fn parse_quantile_alpha(s: &str) -> Result<f64, String> {
    let a = s
        .parse::<f64>()
        .map_err(|_| "alpha 必须是数字".to_string())?;
    if !a.is_finite() || !(0.0..0.5).contains(&a) {
        return Err("alpha 必须满足 0 <= alpha < 0.5".to_string());
    }
    Ok(a)
}

fn parse_stock(s: &str) -> Result<[usize; 6], String> {
    let values = s
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .map_err(|_| "stock 必须包含 6 个非负整数".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    values
        .try_into()
        .map_err(|_| "stock 必须恰好包含 6 个数量，对应 1-6 级".to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_stock;

    #[test]
    fn parses_six_level_stock_vector() {
        assert_eq!(parse_stock("200,2,0,0,0,3"), Ok([200, 2, 0, 0, 0, 3]));
    }

    #[test]
    fn rejects_stock_vector_with_wrong_length() {
        assert!(parse_stock("200,2,0").is_err());
    }

    #[test]
    fn rejects_negative_stock_value() {
        assert!(parse_stock("200,-2,0,0,0,3").is_err());
    }
}

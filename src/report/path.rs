pub fn build_target_cost_run_tag(
    target: usize,
    trials: usize,
    threads: usize,
    pity_enabled: bool,
    seed: Option<u64>,
) -> String {
    let pity = if pity_enabled { "on" } else { "off" };
    let seed_part = match seed {
        Some(v) => format!("s{}", v),
        None => "srand".to_string(),
    };

    format!(
        "t{}_n{}_th{}_p{}_{}",
        target, trials, threads, pity, seed_part
    )
}

pub fn build_stock_drain_run_tag(
    stock: [usize; 6],
    trials: usize,
    threads: usize,
    pity_enabled: bool,
    seed: Option<u64>,
) -> String {
    let stock = stock.map(|count| count.to_string()).join("-");
    let pity = if pity_enabled { "on" } else { "off" };
    let seed = seed.map_or_else(|| "rand".to_string(), |value| value.to_string());

    format!(
        "stock_{}_n{}_th{}_p{}_s{}",
        stock, trials, threads, pity, seed
    )
}

pub fn resolve_output_path(flag: Option<Option<String>>, default_path: &str) -> Option<String> {
    match flag {
        None => None,
        Some(None) => Some(default_path.to_string()),
        Some(Some(path)) => {
            if path == "__AUTO__" || path.trim().is_empty() {
                Some(default_path.to_string())
            } else {
                Some(path)
            }
        }
    }
}

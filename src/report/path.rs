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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_target_and_stock_run_tags() {
        assert_eq!(
            build_target_cost_run_tag(7, 100_000, 4, true, Some(123)),
            "t7_n100000_th4_pon_s123"
        );
        assert_eq!(
            build_stock_drain_run_tag([200, 2, 0, 0, 0, 3], 10_000, 8, false, None),
            "stock_200-2-0-0-0-3_n10000_th8_poff_srand"
        );
    }

    #[test]
    fn resolves_optional_output_paths() {
        let default_path = "default.csv";

        assert_eq!(resolve_output_path(None, default_path), None);
        assert_eq!(
            resolve_output_path(Some(None), default_path).as_deref(),
            Some(default_path)
        );
        assert_eq!(
            resolve_output_path(Some(Some("__AUTO__".to_string())), default_path).as_deref(),
            Some(default_path)
        );
        assert_eq!(
            resolve_output_path(Some(Some("  ".to_string())), default_path).as_deref(),
            Some(default_path)
        );
        assert_eq!(
            resolve_output_path(Some(Some("custom.csv".to_string())), default_path).as_deref(),
            Some("custom.csv")
        );
    }
}

pub struct TargetCostSimulationResult {
    pub trials: usize,
    pub threads: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub ci95_low: f64,
    pub ci95_high: f64,
    pub min: usize,
    pub max: usize,
    pub p50: usize,
    pub p90: usize,
    pub p95: usize,
    pub samples: Vec<usize>,
}

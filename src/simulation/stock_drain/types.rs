#[derive(Clone, Copy, Debug)]
pub struct StockDrainRequest {
    pub stock: [usize; 6],
    pub trials: usize,
    pub requested_threads: Option<usize>,
    pub seed: Option<u64>,
    pub enable_pity: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct StockDrainTrialRequest {
    pub stock: [usize; 6],
    pub seed: Option<u64>,
    pub enable_pity: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StockDrainTrialResult {
    pub pets: [usize; 7],
    pub pity: [usize; 6],
    pub pity_used: [usize; 6],
}

#[derive(Debug, PartialEq, Eq)]
pub struct StockDrainSimulationResult {
    pub trials: usize,
    pub threads: usize,
    pub samples: Vec<StockDrainTrialResult>,
}

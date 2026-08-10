pub struct GameState {
    // pets[n] 表示当前n级宠物数量
    pub pets: [usize; 8],

    // pity[n] 表示n级宠物保底进度 B_n
    pub pity: [usize; 8],

    // 累计消耗一级宠物蛋数量
    pub c1: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            pets: [0; 8],
            pity: [0; 8],
            c1: 0,
        }
    }
}

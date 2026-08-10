// 合成成功概率
// 下标对应等级:
// P[1] = 1级 -> 2级
// P[6] = 6级 -> 7级
pub const P: [f64; 7] = [0.0, 0.95, 0.80, 0.65, 0.45, 0.20, 0.05];

// 保底阈值
pub fn pity_threshold(level: usize) -> usize {
    5 * (level - 1)
}

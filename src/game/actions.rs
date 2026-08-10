use rand::{Rng, RngExt};

use super::rules::{P, pity_threshold};
use super::state::GameState;

// 开一个一级宠物蛋
pub fn open_egg(state: &mut GameState) {
    state.c1 += 1;
    state.pets[1] += 1;
}

// 查找当前是否存在可以合成的等级
pub fn find_merge_level(state: &GameState, target: usize) -> Option<usize> {
    (1..target).find(|&level| state.pets[level] >= 2)
}

// 执行一次合成
pub fn try_merge(state: &mut GameState, level: usize, rng: &mut impl Rng) {
    // 消耗两个低级宠物
    state.pets[level] -= 2;

    if rng.random_bool(P[level]) {
        // 成功
        state.pets[level + 1] += 1;
    } else {
        // 失败返还一个
        state.pets[level] += 1;

        // 增加下一级保底进度
        state.pity[level + 1] += 1;
    }
}

// 检查所有等级保底
pub fn check_pity(state: &mut GameState) -> [usize; 8] {
    let mut redeemed = [0usize; 8];

    for level in 2..=7 {
        if state.pity[level] == pity_threshold(level) {
            // 消耗保底次数
            state.pity[level] = 0;

            // 获得一个对应等级宠物
            state.pets[level] += 1;
            redeemed[level] += 1;
        }
    }

    redeemed
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{RngExt, SeedableRng};

    fn rng_for_outcome(level: usize, want_success: bool) -> StdRng {
        for seed in 0_u64..100_000 {
            let mut probe = StdRng::seed_from_u64(seed);
            if probe.random_bool(P[level]) == want_success {
                return StdRng::seed_from_u64(seed);
            }
        }

        panic!("未在搜索范围内找到满足条件的随机种子");
    }

    #[test]
    fn merge_success_keeps_pity_and_upgrades_pet() {
        let mut state = GameState::new();
        state.pets[3] = 2;
        state.pity[4] = 7;

        let mut rng = rng_for_outcome(3, true);
        try_merge(&mut state, 3, &mut rng);

        assert_eq!(state.pets[3], 0);
        assert_eq!(state.pets[4], 1);
        assert_eq!(state.pity[4], 7);
    }

    #[test]
    fn merge_failure_refunds_one_and_increments_pity() {
        let mut state = GameState::new();
        state.pets[5] = 2;
        state.pity[6] = 3;

        let mut rng = rng_for_outcome(5, false);
        try_merge(&mut state, 5, &mut rng);

        assert_eq!(state.pets[5], 1);
        assert_eq!(state.pets[6], 0);
        assert_eq!(state.pity[6], 4);
    }

    #[test]
    fn pity_exchange_resets_counter_and_grants_pet_without_consumption() {
        let mut state = GameState::new();
        state.pets[6] = 2;
        state.pity[6] = pity_threshold(6);

        let redeemed = check_pity(&mut state);

        assert_eq!(state.pity[6], 0);
        // Existing pets remain; pity exchange grants one extra pet.
        assert_eq!(state.pets[6], 3);
        assert_eq!(redeemed[6], 1);
    }

    #[test]
    fn pity_does_not_trigger_below_threshold() {
        let mut state = GameState::new();
        state.pets[7] = 1;
        state.pity[7] = pity_threshold(7) - 1;

        let redeemed = check_pity(&mut state);

        assert_eq!(state.pity[7], pity_threshold(7) - 1);
        assert_eq!(state.pets[7], 1);
        assert_eq!(redeemed, [0; 8]);
    }
}

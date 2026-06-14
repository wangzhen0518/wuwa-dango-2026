use std::{collections::HashMap, hash::Hash};

use rand::seq::SliceRandom;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    dangos::{Dango, DangoKind, Run, is_budawang, sort_dangos},
    track::{Map, TRACK_LEN, Track, init_map, init_track, show_track},
    utils::{MyRng, gen_rng},
};

#[derive(Debug)]
pub struct GameState {
    // rng: ThreadRng,
    pub rng: MyRng,
    pub map: Map,
    pub dangos: Vec<Dango>,
    pub track: Track,
    pub before_run_dangos: Vec<Dango>,
    pub after_run_dangos: Vec<Dango>,
    pub budawang: Dango,
    pub round: usize,
}

impl GameState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rng: MyRng,
        map: Map,
        dangos: Vec<Dango>,
        track: Track,
        before_run_dangos: Vec<Dango>,
        after_run_dangos: Vec<Dango>,
        budawang: Dango,
        round: usize,
    ) -> Self {
        Self {
            rng,
            map,
            dangos,
            track,
            before_run_dangos,
            after_run_dangos,
            budawang,
            round,
        }
    }
}

pub fn init_game(init_rng: Option<MyRng>) -> GameState {
    let mut rng = init_rng.unwrap_or_else(gen_rng);

    let map = init_map();

    // TODO 从 json 中读取当前 game 的团子
    let budawang = Dango::default_budawang();
    let sigrika = Dango::default_sigrika();
    let mut dangos = vec![
        Dango::default_denia(),
        sigrika.clone(),
        Dango::default_hiyuki(),
        Dango::default_cartethyia(),
        Dango::default_phoebe(),
        Dango::default_luukherssen(),
    ];
    let before_run_dangos = vec![sigrika];
    let after_run_dangos = vec![];

    dangos.shuffle(&mut rng);
    // 根据前进先后顺序更新纵向位置坐标，数组末尾最后一个行动，坐标为 0
    dangos
        .iter_mut()
        .rev()
        .enumerate()
        .for_each(|(idx, dango)| dango.set_pos((0, idx)));
    let track = init_track(&dangos);

    GameState::new(
        rng,
        map,
        dangos,
        track,
        before_run_dangos,
        after_run_dangos,
        budawang,
        0,
    )
}

/// @return: (结束时比赛状态, 布大王团子)
pub fn one_game(first_half_finish_state: Option<GameState>, init_rng: Option<MyRng>) -> GameState {
    let from_beginning = first_half_finish_state.is_none(); // 是否从头开始
    let GameState {
        mut rng,
        map,
        mut dangos,
        mut track,
        before_run_dangos,
        mut after_run_dangos,
        budawang,
        round: _,
    } = first_half_finish_state.unwrap_or_else(|| init_game(init_rng));

    if !from_beginning {
        dangos.iter_mut().for_each(|dango| {
            dango.reset(); // 重置 dango 的部分属性
            dango.increase_target_arrive_count(); // 增加 dango 需要到达终点的次数
        });
    }

    show_track(0, &dangos, &track, &map);

    // 4. 循环 run，直到有团子到达终点
    let mut round = 0;
    let mut arrived = false;
    'GameLoop: while !arrived {
        round += 1;

        // 布大王第三轮开始行动
        if round == 3 {
            budawang.set_pos((TRACK_LEN - 1, 0)); // budawang 的 pos 为上一轮结束时的位置，需要清理
            dangos.push(budawang.clone());
            after_run_dangos.push(budawang.clone());
            track[TRACK_LEN - 1].insert(0, budawang.clone());
            track[TRACK_LEN - 1]
                .iter_mut()
                .enumerate()
                .skip(1)
                .for_each(|(idx, dango)| dango.set_pos((TRACK_LEN - 1, idx)));
        }

        if round != 1 || !from_beginning {
            dangos.shuffle(&mut rng);

            for dango in before_run_dangos.iter() {
                dango.before_run(&dangos, &mut track);
            }
        }

        // 按行动顺序，先全部 roll，再依次 step
        for dango in dangos.iter_mut() {
            dango.roll(&mut rng);
        }

        for dango in dangos.iter() {
            arrived = dango.step(&dangos, &mut track, &map, &mut rng);
            if arrived {
                break 'GameLoop;
            }
        }

        for dango in after_run_dangos.iter_mut() {
            dango.after_run(&dangos, &mut track);
        }
        show_track(round, &dangos, &track, &map);
    }

    // 将布大王从比赛状态中移除
    {
        dangos.retain(|dango| !is_budawang(dango));
        after_run_dangos.retain(|dango| !is_budawang(dango));

        let (x, _) = budawang.get_pos();
        track[x].remove(0);
        // 更新 track[x] 处所有 dango 的坐标
        track[x]
            .iter_mut()
            .enumerate()
            .for_each(|(idx, dango)| dango.set_pos((x, idx)));
    }

    GameState::new(
        rng,
        map,
        dangos,
        track,
        before_run_dangos,
        after_run_dangos,
        budawang,
        round,
    )
}

pub type GameResults = Vec<Vec<DangoKind>>;
pub type RankStatistics = Vec<(Vec<DangoKind>, i64)>;
pub type DangoStatistics = Vec<(DangoKind, i64)>;

/// 大量重复比赛
pub fn simulate_game(n: usize) -> GameResults {
    (0..n)
        .into_par_iter()
        .map(|_i| {
            let mut state = one_game(None, None);
            sort_dangos(&mut state.dangos);
            state.dangos.iter().map(DangoKind::from).collect()
        })
        .collect()
}

/// 统计比赛结果
pub fn statistic_game_result(result: &GameResults) -> (RankStatistics, DangoStatistics) {
    fn insert_dict<Key: Eq + Hash>(key: Key, dict: &mut HashMap<Key, i64>) {
        dict.entry(key).and_modify(|cnt| *cnt += 1).or_insert(1);
    }
    fn extract_dict<Key>(dict: HashMap<Key, i64>) -> Vec<(Key, i64)> {
        let mut v: Vec<_> = dict.into_iter().collect();
        v.sort_unstable_by_key(|(_, cnt)| -cnt);
        v
    }

    let mut rank_stat = HashMap::new();
    let mut dango_stat = HashMap::new();
    for rank in result {
        insert_dict(rank.clone(), &mut rank_stat);
        insert_dict(rank[0], &mut dango_stat);
    }
    let rank_stat = extract_dict(rank_stat);
    let dango_stat = extract_dict(dango_stat);

    (rank_stat, dango_stat)
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::dangos::sort_dangos;

    use super::*;

    // ────────── 完整游戏模拟 ──────────
    fn test_full_game(
        seed: u64,
        half_round_target: usize,
        half_dangos_target: &mut [Dango],
        finish_round_target: usize,
        finish_dangos_target: &mut [Dango],
    ) {
        let rng = StdRng::seed_from_u64(seed).into();

        // 上半场
        let mut half_state = one_game(None, Some(rng));

        assert_eq!(half_state.round, half_round_target);
        assert_eq!(half_state.dangos, half_dangos_target);

        sort_dangos(&mut half_state.dangos);
        sort_dangos(half_dangos_target);
        assert_eq!(half_state.dangos, half_dangos_target);

        // 下半场
        let mut finish_state = one_game(Some(half_state), None);

        assert_eq!(finish_state.round, finish_round_target);
        assert_eq!(finish_state.dangos, finish_dangos_target);

        sort_dangos(&mut finish_state.dangos);
        sort_dangos(finish_dangos_target);
        assert_eq!(finish_state.dangos, finish_dangos_target);
    }

    #[test]
    fn test_full_game_seed_0() {
        let seed = 0;

        // 上半场
        let half_round_target = 9;
        // 卡(30, 1)(0), 菲(28, 0)(0), 陆(30, 0)(0), 西(25, 0)(0), 达(31, 0)(0), 绯(24, 0)(0),
        let mut half_dangos_target = [
            Dango::new_cartethyia(1, (30, 1), 0, 0, 1, false),
            Dango::new_phoebe(1, (28, 0), 0, 0, 1),
            Dango::new_luukherssen(1, (30, 0), 0, 0, 1),
            Dango::new_sigrika(1, (25, 0), 0, 0, 1),
            Dango::new_denia(1, (31, 0), 0, 0, 1, 1),
            Dango::new_hiyuki(1, (24, 0), 0, 0, 1, false),
        ];

        // 下半场
        let finish_round_target = 12;
        // 陆(28, 0)(1), 卡(20, 0)(1), 菲(19, 0)(1), 达(31, 0)(1), 西(0, 0)(1), 绯(23, 0)(1),
        let mut finish_dangos_target = [
            Dango::new_luukherssen(1, (28, 0), 0, 1, 2),
            Dango::new_cartethyia(1, (20, 0), 0, 1, 2, false),
            Dango::new_phoebe(1, (19, 0), 0, 1, 2),
            Dango::new_denia(1, (31, 0), 0, 1, 2, 1),
            Dango::new_sigrika(1, (0, 0), 0, 1, 2),
            Dango::new_hiyuki(1, (23, 0), 0, 1, 2, false),
        ];

        test_full_game(
            seed,
            half_round_target,
            &mut half_dangos_target,
            finish_round_target,
            &mut finish_dangos_target,
        );
    }

    #[test]
    fn test_full_game_seed_1() {
        let seed = 1;

        // 上半场
        let half_round_target = 8;
        // 绯(26, 0)(0), 陆(21, 1)(0), 西(23, 0)(0), 卡(23, 1)(0), 菲(31, 0)(0), 达(21, 0)(0),
        let mut half_dangos_target = [
            Dango::new_hiyuki(1, (26, 0), 0, 0, 1, false),
            Dango::new_luukherssen(1, (21, 1), 0, 0, 1),
            Dango::new_sigrika(1, (23, 0), 0, 0, 1),
            Dango::new_cartethyia(1, (23, 1), 0, 0, 1, false),
            Dango::new_phoebe(1, (31, 0), 0, 0, 1),
            Dango::new_denia(1, (21, 0), 0, 0, 1, 1),
        ];

        // 下半场
        let finish_round_target = 10;
        // 绯(28, 0)(1), 卡(18, 0)(1), 西(31, 0)(1), 菲(31, 1)(1), 陆(16, 0)(1), 达(20, 0)(1),
        let mut finish_dangos_target = [
            Dango::new_hiyuki(1, (28, 0), 0, 1, 2, false),
            Dango::new_cartethyia(1, (18, 0), 0, 1, 2, false),
            Dango::new_sigrika(1, (31, 0), 0, 1, 2),
            Dango::new_phoebe(1, (31, 1), 0, 1, 2),
            Dango::new_luukherssen(1, (16, 0), 0, 1, 2),
            Dango::new_denia(1, (20, 0), 0, 1, 2, 1),
        ];

        test_full_game(
            seed,
            half_round_target,
            &mut half_dangos_target,
            finish_round_target,
            &mut finish_dangos_target,
        );
    }

    #[test]
    fn test_full_game_seed_2() {
        let seed = 2;

        // 上半场
        let half_round_target = 9;
        // 达(29, 1)(0), 卡(29, 0)(0), 西(26, 0)(0), 菲(31, 0)(0), 陆(30, 0)(0), 绯(29, 2)(0),
        let mut half_dangos_target = [
            Dango::new_denia(1, (29, 1), 0, 0, 1, 1),
            Dango::new_cartethyia(1, (29, 0), 0, 0, 1, false),
            Dango::new_sigrika(1, (26, 0), 0, 0, 1),
            Dango::new_phoebe(1, (31, 0), 0, 0, 1),
            Dango::new_luukherssen(1, (30, 0), 0, 0, 1),
            Dango::new_hiyuki(1, (29, 2), 0, 0, 1, false),
        ];

        // 下半场
        let finish_round_target = 10;
        // 菲(28, 0)(1), 西(31, 0)(1), 绯(28, 1)(1), 达(16, 0)(1), 陆(26, 0)(1), 卡(28, 2)(1),
        let mut finish_dangos_target = [
            Dango::new_phoebe(1, (28, 0), 0, 1, 2),
            Dango::new_sigrika(1, (31, 0), 0, 1, 2),
            Dango::new_hiyuki(1, (28, 1), 0, 1, 2, false),
            Dango::new_denia(1, (16, 0), 0, 1, 2, 1),
            Dango::new_luukherssen(1, (26, 0), 0, 1, 2),
            Dango::new_cartethyia(1, (28, 2), 0, 1, 2, false),
        ];

        test_full_game(
            seed,
            half_round_target,
            &mut half_dangos_target,
            finish_round_target,
            &mut finish_dangos_target,
        );
    }

    #[test]
    fn test_full_game_seed_3() {
        let seed = 3;

        // 上半场
        let half_round_target = 10;
        // 陆(31, 0)(0), 达(16, 0)(0), 菲(28, 0)(0), 西(31, 1)(0), 绯(26, 1)(0), 卡(26, 0)(0),
        let mut half_dangos_target = [
            Dango::new_luukherssen(1, (31, 0), 0, 0, 1),
            Dango::new_denia(1, (16, 0), 0, 0, 1, 1),
            Dango::new_phoebe(1, (28, 0), 0, 0, 1),
            Dango::new_sigrika(1, (31, 1), 0, 0, 1),
            Dango::new_hiyuki(1, (26, 1), 0, 0, 1, false),
            Dango::new_cartethyia(1, (26, 0), 0, 0, 1, false),
        ];

        // 下半场
        let finish_round_target = 9;
        // 陆(29, 0)(1), 达(1, 0)(1), 绯(26, 1)(1), 西(31, 0)(1), 菲(20, 0)(1), 卡(26, 0)(0),
        let mut finish_dangos_target = [
            Dango::new_luukherssen(1, (29, 0), 0, 1, 2),
            Dango::new_denia(1, (1, 0), 0, 1, 2, 1),
            Dango::new_hiyuki(1, (26, 1), 0, 1, 2, false),
            Dango::new_sigrika(1, (31, 0), 0, 1, 2),
            Dango::new_phoebe(1, (20, 0), 0, 1, 2),
            Dango::new_cartethyia(1, (26, 0), 0, 0, 2, false),
        ];

        test_full_game(
            seed,
            half_round_target,
            &mut half_dangos_target,
            finish_round_target,
            &mut finish_dangos_target,
        );
    }

    #[test]
    fn test_full_game_seed_4() {
        let seed = 4;

        // 上半场
        let half_round_target = 9;
        // 陆(25, 0)(0), 卡(25, 1)(0), 西(31, 1)(0), 达(23, 0)(0), 菲(31, 0)(0), 绯(26, 0)(0),
        let mut half_dangos_target = [
            Dango::new_luukherssen(1, (25, 0), 0, 0, 1),
            Dango::new_cartethyia(1, (25, 1), 0, 0, 1, false),
            Dango::new_sigrika(1, (31, 1), 0, 0, 1),
            Dango::new_denia(1, (23, 0), 0, 0, 1, 1),
            Dango::new_phoebe(1, (31, 0), 0, 0, 1),
            Dango::new_hiyuki(1, (26, 0), 0, 0, 1, false),
        ];

        // 下半场
        let finish_round_target = 12;
        // 绯(26, 1)(1), 陆(24, 0)(1), 卡(21, 0)(1), 菲(26, 2)(1), 达(31, 0)(1), 西(26, 0)(1),
        let mut finish_dangos_target = [
            Dango::new_hiyuki(1, (26, 1), 0, 1, 2, false),
            Dango::new_luukherssen(1, (24, 0), 0, 1, 2),
            Dango::new_cartethyia(1, (21, 0), 0, 1, 2, false),
            Dango::new_phoebe(1, (26, 2), 0, 1, 2),
            Dango::new_denia(1, (31, 0), 0, 1, 2, 1),
            Dango::new_sigrika(1, (26, 0), 0, 1, 2),
        ];

        test_full_game(
            seed,
            half_round_target,
            &mut half_dangos_target,
            finish_round_target,
            &mut finish_dangos_target,
        );
    }
}

use rand::seq::SliceRandom;

use crate::{
    dangos::{Dango, Run, is_budawang},
    track::{Map, TRACK_LEN, Track, init_map, init_track, show_track},
    utils::{MyRng, gen_rng},
};

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

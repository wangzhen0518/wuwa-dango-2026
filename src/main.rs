use std::fmt::Write;

use rand::{rngs::ThreadRng, seq::SliceRandom};

use crate::{
    dangos::{RefDango, Run, is_budawang},
    track::{Map, TRACK_LEN, Track, init_map, init_track, show_track},
};

mod dangos;
mod track;
mod utils;

#[derive(Debug, Clone)]
pub struct GameState {
    rng: ThreadRng,
    map: Map,
    dangos: Vec<RefDango>,
    track: Track,
    before_run_dangos: Vec<RefDango>,
    after_run_dangos: Vec<RefDango>,
    budawang: RefDango,
    round: usize,
}

// fn one_round() {
// 1. 随机决定团子前进顺序
// 2. 按前进顺序遍历团子
// 3. roll 点确定前进步数
// 4. 根据 roll 点和团子位置关系更新团子状态（多走或者少走 n 格）
// 5. 根据点数和技能前进
// 6. 根据赛道效果和技能前进
// }

impl GameState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rng: ThreadRng,
        map: Map,
        dangos: Vec<RefDango>,
        track: Track,
        before_run_dangos: Vec<RefDango>,
        after_run_dangos: Vec<RefDango>,
        budawang: RefDango,
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

fn init_game() -> GameState {
    let mut rng = rand::rng();

    let map = init_map();

    // TODO 从 json 中读取当前 game 的团子
    let budawang = dangos::new_bu_da_wang();
    let sigrika = dangos::new_sigrika();
    let mut dangos = vec![
        dangos::new_denia(),
        sigrika.clone(),
        dangos::new_hiyuki(),
        dangos::new_cartethyia(),
        dangos::new_phoebe(),
        dangos::new_luuk_herssen(),
    ];
    let before_run_dangos = vec![sigrika];
    let after_run_dangos = vec![];

    dangos.shuffle(&mut rng);
    // 根据前进先后顺序更新纵向位置坐标，数组末尾最后一个行动，坐标为 0
    dangos
        .iter()
        .rev()
        .enumerate()
        .for_each(|(idx, dango)| dango.borrow_mut().set_pos((0, idx)));
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
fn one_game(first_half_finish_state: Option<GameState>) -> GameState {
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
    } = first_half_finish_state.unwrap_or_else(init_game);

    if !from_beginning {
        budawang.borrow_mut().set_pos((TRACK_LEN - 1, 0)); // budawang 的 pos 为上一轮结束时的位置，需要清理
        dangos.iter().for_each(|dango| {
            let mut dango = dango.borrow_mut();
            dango.reset(); // 重置 dango 的部分属性
            dango.increase_target_arrive_count(); // 增加 dango 需要到达终点的次数
        });
    }

    show_track(0, &track);

    // 4. 循环 run，直到有团子到达终点
    let mut round = 0;
    let mut arrived = false;
    'GameLoop: while !arrived {
        round += 1;

        // 布大王第三轮开始行动
        if round == 3 {
            dangos.push(budawang.clone());
            after_run_dangos.push(budawang.clone());
            track[TRACK_LEN - 1].push(budawang.clone());
        }

        if round != 1 || !from_beginning {
            dangos.shuffle(&mut rng);

            for dango in before_run_dangos.iter() {
                dango.borrow_mut().before_run(&mut track);
            }
        }

        for dango in dangos.iter() {
            arrived = dango.borrow_mut().step(&mut track, &map, &mut rng);
            if arrived {
                break 'GameLoop;
            }
        }

        for dango in after_run_dangos.iter() {
            dango.borrow_mut().after_run(&mut track);
        }
        show_track(round, &track);
    }

    // 将布大王从比赛状态中移除
    {
        dangos.retain(|dango| !is_budawang(dango));
        after_run_dangos.retain(|dango| !is_budawang(dango));

        let budawang = budawang.borrow();
        let (x, _) = budawang.get_pos();
        track[x].remove(0);
        // 更新 track[x] 处所有 dango 的坐标
        track[x]
            .iter()
            .enumerate()
            .for_each(|(idx, dango)| dango.borrow_mut().set_pos((x, idx)));
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

fn sort_by_dangos(dangos: &mut [RefDango]) {
    dangos.sort_by(|a, b| {
        let (x_a, y_a) = a.borrow().get_pos();
        let (x_b, y_b) = b.borrow().get_pos();
        if x_a == x_b {
            y_a.cmp(&y_b)
        } else {
            x_a.cmp(&x_b)
        }
    });
    dangos.reverse();
}

fn sort_by_track(track: &Track) -> Vec<RefDango> {
    track
        .iter()
        .rev()
        .flat_map(|point| point.iter().rev())
        .cloned()
        .collect()
}

fn show_rank(dangos: &[RefDango]) {
    let mut rank_info = String::with_capacity(10 * dangos.len());
    for dango in dangos.iter() {
        let dango = dango.borrow();
        let (x, y) = dango.get_pos();
        write!(&mut rank_info, "{}({}, {}), ", dango.shortname(), x, y).unwrap();
    }
    println!("{}", rank_info);
}

fn main() {
    println!("Start first half game");
    let mut half_state = one_game(None);
    show_track(half_state.round, &half_state.track);
    sort_by_dangos(&mut half_state.dangos);
    show_rank(&half_state.dangos);
    show_rank(&sort_by_track(&half_state.track));

    println!("Start second half game");
    let mut finish_state = one_game(Some(half_state));
    show_track(finish_state.round, &finish_state.track);
    sort_by_dangos(&mut finish_state.dangos);
    show_rank(&finish_state.dangos);
    show_rank(&sort_by_track(&finish_state.track));
}

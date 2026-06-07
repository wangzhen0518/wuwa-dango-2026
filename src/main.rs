#![allow(unused)]

use std::fmt::Write;

use ambassador::{Delegate, delegatable_trait_remote};
use rand::{
    SeedableRng, TryRng,
    rngs::{StdRng, ThreadRng},
    seq::SliceRandom,
};

mod dangos;
mod track;
mod utils;

use crate::{
    dangos::{Dango, Run, is_budawang},
    track::{Map, TRACK_LEN, Track, init_map, init_track, show_track},
};

#[delegatable_trait_remote]
trait TryRng {
    type Error: core::error::Error;
    fn try_next_u32(&mut self) -> Result<u32, Self::Error>;
    fn try_next_u64(&mut self) -> Result<u64, Self::Error>;
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Delegate)]
#[delegate(TryRng)]
pub enum MyRng {
    StdRng(Box<StdRng>),
    ThreadRng(ThreadRng),
}

// impl DerefMut for MyRng {}

impl From<StdRng> for MyRng {
    fn from(rng: StdRng) -> Self {
        MyRng::StdRng(Box::new(rng))
    }
}

impl From<ThreadRng> for MyRng {
    fn from(rng: ThreadRng) -> Self {
        MyRng::ThreadRng(rng)
    }
}

pub struct GameState {
    // rng: ThreadRng,
    rng: MyRng,
    map: Map,
    dangos: Vec<Dango>,
    track: Track,
    before_run_dangos: Vec<Dango>,
    after_run_dangos: Vec<Dango>,
    budawang: Dango,
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

fn init_game() -> GameState {
    let args: Vec<_> = std::env::args().collect();
    let mut rng = if args.len() > 1 {
        let seed: u64 = args[1].parse().unwrap();
        println!("seed = {}", seed);
        StdRng::seed_from_u64(seed).into()
    } else {
        rand::rng().into()
    };

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

#[allow(unused)]
fn show_rank(dangos: &[Dango]) {
    let mut rank_info = String::with_capacity(10 * dangos.len());
    for dango in dangos.iter() {
        let (x, y) = dango.get_pos();
        write!(&mut rank_info, "{}({}, {}), ", dango.shortname(), x, y).expect("Write failed");
    }
    println!("{}", rank_info);
}

fn main() {
    if cfg!(debug_assertions) {
        println!("Start first half game");
    }
    let half_state = one_game(None);
    show_track(
        half_state.round,
        &half_state.dangos,
        &half_state.track,
        &half_state.map,
    );
    //sort_by_dangos(&mut half_state.dangos);
    //show_rank(&half_state.dangos);
    //show_rank(&sort_by_track(&half_state.track));

    if cfg!(debug_assertions) {
        println!("Start second half game");
    }
    let mut finish_state = one_game(Some(half_state));
    show_track(
        finish_state.round,
        &finish_state.dangos,
        &finish_state.track,
        &finish_state.map,
    );
    //sort_by_dangos(&mut finish_state.dangos);
    //show_rank(&finish_state.dangos);
    //show_rank(&sort_by_track(&finish_state.track));
}

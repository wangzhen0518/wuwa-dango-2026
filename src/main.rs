// #![allow(unused)]

mod dangos;
mod game;
mod track;
mod utils;

use crate::{
    dangos::{show_dangos, sort_dangos},
    game::{one_game, simulate_game, statistic_game_result},
    track::show_track,
    utils::debug_print,
};

#[allow(dead_code)]
fn demo_game() {
    debug_print("Start first half game");
    let mut half_state = one_game(None, None);
    show_track(
        half_state.round,
        &half_state.dangos,
        &half_state.track,
        &half_state.map,
    );
    show_dangos(&half_state.dangos);
    sort_dangos(&mut half_state.dangos);
    show_dangos(&half_state.dangos);

    debug_print("Start second half game");
    let mut finish_state = one_game(Some(half_state), None);
    show_track(
        finish_state.round,
        &finish_state.dangos,
        &finish_state.track,
        &finish_state.map,
    );
    show_dangos(&finish_state.dangos);
    sort_dangos(&mut finish_state.dangos);
    show_dangos(&finish_state.dangos);
}

#[allow(dead_code)]
fn demo_statistics() {
    let n = 1_000_000;

    let game_results = simulate_game(n);
    let (rank_stat, dango_stat) = statistic_game_result(&game_results);
    for (rank, cnt) in rank_stat.iter().take(10) {
        println!(
            "{:?}: {}",
            rank.iter()
                .map(|dango| dango.shortname())
                .collect::<Vec<_>>(),
            cnt
        );
    }
    for (dango, cnt) in dango_stat.iter().take(10) {
        println!("{:?}: {}", dango.shortname(), cnt);
    }
    // dbg!(&rank_stat[0..10]);
    // dbg!(&dango_stat);

    let rank_total: i64 = rank_stat.iter().map(|(_, cnt)| cnt).sum();
    assert_eq!(rank_total as usize, n);

    let dango_total: i64 = dango_stat.iter().map(|(_, cnt)| cnt).sum();
    assert_eq!(dango_total as usize, n);
}

fn main() {
    demo_statistics();
}

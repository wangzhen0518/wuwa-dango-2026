// #![allow(unused)]

mod dangos;
mod game;
mod track;
mod utils;

use crate::{
    dangos::{show_dangos, sort_dangos},
    game::one_game,
    track::show_track,
    utils::debug_print,
};

fn main() {
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

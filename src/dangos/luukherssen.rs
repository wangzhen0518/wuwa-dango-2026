use std::cell::RefCell;

use rand::Rng;

use crate::{
    dangos::{Dango, Run, impl_run_helper},
    track::{Map, Track},
};

#[derive(Debug, Clone)]
pub struct LuukHerssen {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl LuukHerssen {
    pub fn new() -> Self {
        Self {
            n: 0,
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
        }
    }
}

impl Run for RefCell<LuukHerssen> {
    impl_run_helper!();

    fn accelerate_step(&self) -> usize {
        4
    }

    fn decelerate_step(&self) -> usize {
        2
    }
}

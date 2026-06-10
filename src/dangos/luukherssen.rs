use std::{cell::RefCell, rc::Rc};

use crate::dangos::{Run, impl_run_helper};

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

impl Default for LuukHerssen {
    fn default() -> Self {
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

pub fn new_luukherssen(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
) -> Rc<RefCell<LuukHerssen>> {
    Rc::new(RefCell::new(LuukHerssen {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
    }))
}

pub fn default_luukherssen() -> Rc<RefCell<LuukHerssen>> {
    Rc::new(RefCell::new(LuukHerssen::default()))
}

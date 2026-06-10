use std::{cell::RefCell, rc::Rc};

use rand::{Rng, RngExt};

use crate::{
    dangos::{Dango, Run, impl_run_helper},
    track::{Map, Track},
};

const EXTRA_ADVANCE_PROB: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct Phoebe {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Default for Phoebe {
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

impl Run for RefCell<Phoebe> {
    impl_run_helper!();

    fn step<R>(&self, _dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        if rng.random_bool(EXTRA_ADVANCE_PROB) {
            self.borrow_mut().extra += 1;
        }

        self.make_step(track, map, rng)
    }
}

pub fn new_phoebe(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
) -> Rc<RefCell<Phoebe>> {
    Rc::new(RefCell::new(Phoebe {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
    }))
}

pub fn default_phoebe() -> Rc<RefCell<Phoebe>> {
    Rc::new(RefCell::new(Phoebe::default()))
}

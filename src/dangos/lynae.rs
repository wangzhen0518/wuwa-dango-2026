use std::{cell::RefCell, rc::Rc};

use rand::{Rng, RngExt};

use crate::{
    dangos::{Dango, Run, impl_run_helper},
    track::{Map, Track},
};

const DOUBLE_PROB: f64 = 0.6;
const STAY_PROB: f64 = 0.2;

#[derive(Debug, Clone)]
pub struct Lynae {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Default for Lynae {
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

impl Run for RefCell<Lynae> {
    impl_run_helper!();

    fn step<R>(&self, _dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let prob: f64 = rng.random();

        if prob < DOUBLE_PROB {
            self.borrow_mut().n *= 2;
        }

        if prob < 1.0 - STAY_PROB {
            self.make_step(track, map, rng)
        } else {
            false
        }
    }
}

pub fn new_lynae(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
) -> Rc<RefCell<Lynae>> {
    Rc::new(RefCell::new(Lynae {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
    }))
}

pub fn default_lynae() -> Rc<RefCell<Lynae>> {
    Rc::new(RefCell::new(Lynae::default()))
}

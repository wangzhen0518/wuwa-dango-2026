use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::{
    dangos::{Dango, Run, impl_run_helper},
    track::{Map, Track},
};

#[derive(Debug, Clone)]
pub struct Denia {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    last_dice: usize,
}

impl Default for Denia {
    fn default() -> Self {
        Self {
            n: 0,
            last_dice: 0,
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
        }
    }
}

impl Run for RefCell<Denia> {
    impl_run_helper!();

    fn reset(&self) {
        let mut self_mut_inner = self.borrow_mut();
        self_mut_inner.last_dice = 0;
        self_mut_inner.extra = 0;
        self_mut_inner.n = 0;
    }

    fn step<R>(&self, _dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.get_n();

        let mut self_mut_inner = self.borrow_mut();
        if n == self_mut_inner.last_dice {
            self_mut_inner.extra += 2;
        }
        self_mut_inner.last_dice = n;
        drop(self_mut_inner);

        self.make_step(track, map, rng)
    }
}

pub fn new_denia(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    last_dice: usize,
) -> Rc<RefCell<Denia>> {
    Rc::new(RefCell::new(Denia {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
        last_dice,
    }))
}

pub fn default_denia() -> Rc<RefCell<Denia>> {
    Rc::new(RefCell::new(Denia::default()))
}

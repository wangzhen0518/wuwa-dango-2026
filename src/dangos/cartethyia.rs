use std::{cell::RefCell, rc::Rc};

use rand::{Rng, RngExt};

use crate::{
    dangos::{Dango, Run, impl_run_helper},
    track::{Map, Track},
};

const EXTRA_ADVANCE_PROB: f64 = 0.6;
#[derive(Debug, Clone)]
pub struct Cartethyia {
    n: usize,
    /// 是否成为过最后一名
    has_been_last: bool,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Cartethyia {
    pub fn new() -> Self {
        Self {
            n: 0,
            has_been_last: false,
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
        }
    }

    fn is_last(&self, dangos: &[Dango]) -> bool {
        // 收集除自己和布大王以外、落后于自己的团子
        let after_self_dangos: Vec<_> = dangos
            .iter()
            .filter(|dango| {
                !matches!(dango, Dango::BuDaWang(_) | Dango::Cartethyia(_))
                    && dango
                        .get_arrive_count()
                        .cmp(&self.arrive_count)
                        .then(dango.get_pos().cmp(&self.pos))
                        .is_lt()
            })
            .cloned()
            .collect();

        if cfg!(debug_assertions) {
            #[allow(clippy::needless_bool)]
            if after_self_dangos.is_empty() {
                true
            } else {
                false
            }
        } else {
            after_self_dangos.is_empty()
        }
    }
}

impl Run for RefCell<Cartethyia> {
    impl_run_helper!();

    fn reset(&self) {
        let mut self_mut_inner = self.borrow_mut();
        self_mut_inner.has_been_last = false;
        self_mut_inner.extra = 0;
        self_mut_inner.n = 0;
    }

    fn step<R>(&self, dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let mut self_mut_inner = self.borrow_mut();
        if self_mut_inner.has_been_last && rng.random_bool(EXTRA_ADVANCE_PROB) {
            self_mut_inner.extra += 2;
        }
        drop(self_mut_inner);

        let arrived = self.make_step(track, map, rng);

        let mut self_mut_inner = self.borrow_mut();
        if !arrived && !self_mut_inner.has_been_last {
            self_mut_inner.has_been_last = self_mut_inner.is_last(dangos);
        }

        arrived
    }
}

pub fn new_cartethyia() -> Dango {
    Dango::Cartethyia(Rc::new(RefCell::new(Cartethyia::new())))
}

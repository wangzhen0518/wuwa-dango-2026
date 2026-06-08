use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::{
    dangos::{Dango, Run, impl_run_helper, sort_dangos},
    track::{Map, Track},
};

#[derive(Debug, Clone)]
pub struct Sigrika {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Sigrika {
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

impl Run for RefCell<Sigrika> {
    impl_run_helper!();

    fn before_run(&self, dangos: &[Dango], track: &mut Track) {
        let self_inner = self.borrow();
        // 收集除布大王外，领先于自己的团子
        let mut ahead_dangos: Vec<_> = dangos
            .iter()
            .filter(|dango| {
                !matches!(dango, Dango::BuDaWang(_) | Dango::Sigrika(_))
                    && dango
                        .get_arrive_count()
                        .cmp(&self_inner.arrive_count)
                        .then(dango.get_pos().cmp(&self_inner.pos))
                        .is_gt()
            })
            .cloned()
            .collect();

        if !ahead_dangos.is_empty() {
            sort_dangos(&mut ahead_dangos);
            ahead_dangos.iter_mut().rev().take(2).for_each(|dango| {
                let target_extra = dango.get_extra() - 1;
                dango.set_extra(target_extra);
            });
        }
    }
}

pub fn new_sigrika() -> Dango {
    Dango::Sigrika(Rc::new(RefCell::new(Sigrika::new())))
}

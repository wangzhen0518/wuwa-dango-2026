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
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    /// 是否成为过最后一名
    has_been_last: bool,
}

impl Cartethyia {
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
            .collect();

        after_self_dangos.is_empty()
    }
}

impl Default for Cartethyia {
    fn default() -> Self {
        Self {
            n: 0,
            has_been_last: false,
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
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

pub fn new_cartethyia(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    has_been_last: bool,
) -> Rc<RefCell<Cartethyia>> {
    Rc::new(RefCell::new(Cartethyia {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
        has_been_last,
    }))
}

pub fn default_cartethyia() -> Rc<RefCell<Cartethyia>> {
    Rc::new(RefCell::new(Cartethyia::default()))
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::dangos::tests::*;

    use super::*;

    #[test]
    fn test_is_last_true_when_behind_all() {
        let cartethyia = default_cartethyia();
        let denia = Dango::default_denia();
        let hiyuki = Dango::default_hiyuki();
        hiyuki.set_pos((2, 0));
        denia.set_pos((1, 0));
        cartethyia.set_pos((0, 0));
        let dangos = [denia, hiyuki, cartethyia.clone().into()];

        assert!(
            cartethyia.borrow().is_last(&dangos),
            "should be last when behind all"
        );
    }

    #[test]
    fn test_is_last_false_when_not_last() {
        let cartethyia = default_cartethyia();
        let denia = Dango::default_denia();
        let hiyuki = Dango::default_hiyuki();
        cartethyia.set_pos((2, 0));
        denia.set_pos((1, 0));
        hiyuki.set_pos((0, 0));
        let dangos = [denia, hiyuki, cartethyia.clone().into()];

        assert!(!cartethyia.borrow().is_last(&dangos), "should not be last");
    }

    #[test]
    fn test_flag_gets_set_when_last() {
        let mut rng = StdRng::seed_from_u64(42);

        let map = dummy_map();
        let mut track = dummy_track_no_dangos();

        let cartethyia = default_cartethyia();
        let denia = Dango::default_denia();
        let hiyuki = Dango::default_hiyuki();
        let dangos = [denia.clone(), hiyuki.clone(), cartethyia.clone().into()];

        hiyuki.set_pos((3, 0));
        denia.set_pos((2, 0));
        cartethyia.set_pos((0, 0));

        track[3].push(hiyuki.clone());
        track[2].push(denia.clone());
        track[0].push(cartethyia.clone().into());

        cartethyia.set_n(1);
        cartethyia.step(&dangos, &mut track, &map, &mut rng);

        assert!(
            cartethyia.borrow().has_been_last,
            "flag should be set after being last"
        );
    }
}

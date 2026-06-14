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
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
            last_dice: 0,
        }
    }
}

impl Run for RefCell<Denia> {
    impl_run_helper!();

    fn reset(&self) {
        let mut self_mut_inner = self.borrow_mut();
        self_mut_inner.extra = 0;
        self_mut_inner.n = 0;
        self_mut_inner.last_dice = 0;
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

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::dangos::tests::*;

    use super::*;

    #[test]
    fn test_denia_consecutive_dice_bonus() {
        let mut rng = StdRng::seed_from_u64(0);

        let map = dummy_map();
        let mut track = dummy_track_no_dangos();

        let denia = default_denia();
        let dangos = [denia.clone().into()];

        denia.set_pos((0, 0));
        track[0].push(denia.clone().into());

        // 第一次 step 设定 last_dice
        let target_n = 2;
        denia.set_n(target_n);
        denia.step(&dangos, &mut track, &map, &mut rng);

        // 手动再 roll 一次
        denia.set_n(target_n);
        denia.step(&dangos, &mut track, &map, &mut rng);

        // Denia::step: if n == last_dice, extra += 2
        let (x, _) = denia.get_pos();
        assert_eq!(x, target_n + target_n + 2);
        assert_eq!(denia.get_extra(), 0);
    }
}

use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::{
    dangos::{Dango, Run, impl_run_helper},
    track::{Map, Track},
};

#[derive(Debug, Clone)]
pub struct Chisa {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Default for Chisa {
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

impl Run for RefCell<Chisa> {
    impl_run_helper!();

    fn step<R>(&self, dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        if self.get_n()
            == dangos
                .iter()
                .map(|dango| dango.get_n())
                .min()
                .expect("Always have dangos.")
        {
            self.borrow_mut().extra += 2;
        }

        self.make_step(track, map, rng)
    }
}

pub fn new_chisa(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
) -> Rc<RefCell<Chisa>> {
    Rc::new(RefCell::new(Chisa {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
    }))
}

pub fn default_chisa() -> Rc<RefCell<Chisa>> {
    Rc::new(RefCell::new(Chisa::default()))
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

        let denia = Dango::default_denia();
        let chisa = default_chisa();
        let dangos = [denia.clone(), chisa.clone().into()];

        denia.set_pos((0, 0));
        chisa.set_pos((1, 0));
        track[0].push(denia.clone());
        track[1].push(chisa.clone().into());

        denia.set_n(3);
        chisa.set_n(1);

        chisa.step(&dangos, &mut track, &map, &mut rng);

        let (x, _) = chisa.get_pos();
        assert_eq!(x, 1 + 1 + 2);
        assert_eq!(chisa.get_extra(), 0);
    }
}

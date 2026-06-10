use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::{
    dangos::{Dango, Run, has_budawang, impl_run_helper, is_budawang},
    track::{Map, Track},
};

#[derive(Debug, Clone)]
pub struct Hiyuki {
    n: usize,

    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    /// 是否遇到过布大王
    meeted: bool,
}

impl Default for Hiyuki {
    fn default() -> Self {
        Self {
            n: 0,
            meeted: false,
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
        }
    }
}

impl Run for RefCell<Hiyuki> {
    impl_run_helper!();

    fn reset(&self) {
        let mut self_inner = self.borrow_mut();
        self_inner.meeted = false;
        self_inner.extra = 0;
        self_inner.n = 0;
    }

    fn step<R>(&self, _dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let (old_x, _) = self.get_pos();

        // 绯雪上轮行动后至此轮行动前，被布大王经过
        let mut self_mut_inner = self.borrow_mut();
        if !self_mut_inner.meeted && is_budawang(&track[old_x][0]) {
            self_mut_inner.meeted = true;
        }
        self_mut_inner.extra = self_mut_inner.meeted as isize;
        drop(self_mut_inner); // self.make_step 中会进行 borrow，需要事先释放 borrow

        let arrived = self.make_step(track, map, rng);

        // 移动结束后，是否遇到过布大王
        let mut self_mut_inner = self.borrow_mut();
        if !arrived && !self_mut_inner.meeted {
            let (new_x, _) = self_mut_inner.pos;
            // 没有考虑绯雪越过终点绕一圈之后，仍 new_x > old_x 的情况，应该不会发生这种情况
            if new_x > old_x {
                self_mut_inner.meeted = has_budawang(&track[old_x + 1..=new_x]);
            } else {
                self_mut_inner.meeted =
                    has_budawang(&track[old_x + 1..]) || has_budawang(&track[..=new_x]);
            }
        }

        arrived
    }
}

pub fn new_hiyuki(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    meeted: bool,
) -> Rc<RefCell<Hiyuki>> {
    Rc::new(RefCell::new(Hiyuki {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
        meeted,
    }))
}

pub fn default_hiyuki() -> Rc<RefCell<Hiyuki>> {
    Rc::new(RefCell::new(Hiyuki::default()))
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::dangos::tests::*;

    use super::*;

    #[test]
    fn test_hiyuki_meet_budawang_bonus() {
        const N: usize = 2;
        const BUDAWANG_POS: usize = 5;
        const HIYUKI_POS: usize = 4;

        let mut rng = StdRng::seed_from_u64(0);

        let map = dummy_map();
        let mut track = dummy_track_no_dangos();

        let budawang = Dango::default_budawang();
        let hiyuki = default_hiyuki();
        let dangos = [budawang.clone(), hiyuki.clone().into()];

        budawang.set_pos((5, 0));
        hiyuki.set_pos((4, 0));

        track[BUDAWANG_POS].push(budawang.clone());
        track[HIYUKI_POS].push(hiyuki.clone().into());

        hiyuki.set_n(N);
        hiyuki.step(&dangos, &mut track, &map, &mut rng);
        assert!(hiyuki.borrow().meeted, "Should have meeted BuDaWang");

        hiyuki.set_n(N);
        hiyuki.step(&dangos, &mut track, &map, &mut rng);

        let (x, _) = hiyuki.get_pos();
        assert_eq!(x, HIYUKI_POS + N * 2 + 1);
    }
}

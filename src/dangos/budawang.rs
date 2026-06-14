use std::{cell::RefCell, rc::Rc};

use rand::{
    Rng,
    seq::{IndexedRandom, SliceRandom},
};

use crate::{
    dangos::{Dango, Run},
    track::{Map, PointType, TRACK_LEN, Track},
    utils::split_first,
};

static BUDAWANG_DICE: [usize; 6] = [1, 2, 3, 4, 5, 6];

#[derive(Debug, Clone)]
pub struct BuDaWang {
    n: usize,
    pos: (usize, usize),
}

impl BuDaWang {
    fn leave_last_dango(&self, dangos: &[Dango]) -> bool {
        let (x, _) = self.pos;

        // 除布大王以外的最后一名团子
        let last_dango = dangos
            .iter()
            .filter(|dango| !matches!(dango, Dango::BuDaWang(_)))
            .min()
            .expect("Always has dangos");

        let (last_x, _) = last_dango.get_pos();

        last_x > x
    }
}

impl Default for BuDaWang {
    fn default() -> Self {
        Self {
            n: 0,
            pos: (TRACK_LEN - 1, 0),
        }
    }
}

impl Run for RefCell<BuDaWang> {
    fn roll<R>(&self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let n = *BUDAWANG_DICE.choose(rng).expect("Roll failed");
        self.borrow_mut().n = n;
    }

    fn get_n(&self) -> usize {
        self.borrow().n
    }

    fn set_n(&self, n: usize) {
        self.borrow_mut().n = n;
    }

    fn get_extra(&self) -> isize {
        0
    }

    fn set_extra(&self, _extra: isize) {}

    fn get_pos(&self) -> (usize, usize) {
        self.borrow().pos
    }

    fn set_pos(&self, pos: (usize, usize)) {
        self.borrow_mut().pos = pos
    }

    fn get_arrive_count(&self) -> usize {
        0
    }

    fn increase_arrive_count(&self) {}

    fn get_target_arrive_count(&self) -> usize {
        0
    }

    fn increase_target_arrive_count(&self) {}

    fn step<R>(&self, _dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.get_n();

        let (x, _) = self.get_pos(); // y == 0 恒成立

        // 计算目标行，限制在有效范围
        let mut target_x = x.saturating_sub(n);

        let (self_dango, mut tail) = split_first(std::mem::take(&mut track[x]));
        let (left, right) = track.split_at_mut(target_x + 1);
        // 布大王在底层
        left[target_x].insert(0, self_dango);
        // 布大王将经过的格子上的团子都移动到 target_x 处
        for right_i in right
            .iter_mut()
            .take(x - target_x - 1)
            .filter(|point| !point.is_empty())
        {
            left[target_x].append(right_i);
        }
        track[target_x].append(&mut tail);

        match map[target_x] {
            PointType::Common => {}
            PointType::Accelerate => {
                // 由于布大王从终点向起点移动，所以必然已经经过 target_x + 1 处的格子，所以直接移动即可
                let (left, right) = track.split_at_mut(target_x + 1);
                right[0].append(&mut left[target_x]);
                target_x += 1;
            }
            PointType::Decelerate => {
                let (self_dango, mut tail) = split_first(std::mem::take(&mut track[target_x]));
                track[target_x - 1].insert(0, self_dango);
                track[target_x - 1].append(&mut tail);
                target_x -= 1;
            }
            PointType::Hole => {
                let (_, right) = track[target_x].split_at_mut(1);
                right.shuffle(rng);
            }
        }

        self.set_pos((target_x, 0));
        track[target_x]
            .iter_mut()
            .enumerate()
            .skip(1)
            .for_each(|(idx, dango)| dango.set_pos((target_x, idx)));

        false
    }

    fn after_run(&self, dangos: &[Dango], track: &mut Track) {
        if self.borrow().leave_last_dango(dangos) {
            let (x, _) = self.get_pos();

            // remove(0) 而不是 pop，因为可能已经与最后一名分离了，但是可能有团子已经跑了一圈来到布大王上方了
            let self_dango = track[x].remove(0);
            track[x]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, dango)| dango.set_pos((x, idx)));

            self.set_pos((TRACK_LEN - 1, 0));
            track[TRACK_LEN - 1].insert(0, self_dango);
            track[TRACK_LEN - 1]
                .iter_mut()
                .enumerate()
                .skip(1)
                .for_each(|(idx, dango)| dango.set_pos((TRACK_LEN - 1, idx)));
        }
    }
}

pub fn new_budawang(n: usize, pos: (usize, usize)) -> Rc<RefCell<BuDaWang>> {
    Rc::new(RefCell::new(BuDaWang { n, pos }))
}

pub fn default_budawang() -> Rc<RefCell<BuDaWang>> {
    Rc::new(RefCell::new(BuDaWang::default()))
}

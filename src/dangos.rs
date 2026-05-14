use std::{cell::RefCell, rc::Rc};

use ambassador::{Delegate, delegatable_trait};
use rand::{
    Rng, RngExt,
    seq::{IndexedRandom, SliceRandom},
};

use crate::{
    track::{Map, Point, PointType, TRACK_LEN, Track},
    utils::split_first,
};

static COMMON_DICE: [usize; 6] = [1, 1, 2, 2, 3, 3];

// #[delegatable_trait]

#[delegatable_trait]
pub trait Run {
    fn roll<R>(&self, rng: &mut R) -> usize
    where
        R: Rng + ?Sized,
    {
        *COMMON_DICE.choose(rng).unwrap()
    }

    fn get_extra(&self) -> isize;
    fn set_extra(&mut self, extra: isize);

    fn get_pos(&self) -> (usize, usize);
    fn set_pos(&mut self, pos: (usize, usize));

    fn accelerate_step(&self) -> usize {
        1
    }

    fn decelerate_step(&self) -> usize {
        1
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized;

    fn make_step<R>(&mut self, n: usize, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = n.saturating_add_signed(self.get_extra()).max(1);
        let (x, y) = self.get_pos();

        // 移除尾部元素
        let mut tail = track[x].split_off(y);

        // 计算目标行，限制在有效范围
        let mut target_x = (x + n).min(track.len() - 1);
        // 将尾部元素追加到目标行
        let mut target_y = track[target_x].len();
        track[target_x].append(&mut tail);

        match map[target_x] {
            PointType::Common => {}
            PointType::Accelerate => {
                let acc = self.accelerate_step();
                let new_x = (target_x + acc).min(track.len() - 1);
                let (left, right) = track.split_at_mut(new_x);

                target_y += right[0].len();
                right[0].append(&mut left[target_x]);
                target_x = new_x;
            }
            PointType::Decelerate => {
                let dec = self.decelerate_step();
                let new_x = target_x.saturating_sub(dec);
                let (left, right) = track.split_at_mut(target_x);

                target_y += left[new_x].len();
                left[new_x].append(&mut right[0]);
                target_x = new_x
            }
            PointType::Hole => {
                // 打标记以便 shuffle 后找到 self 的新位置
                self.set_pos((usize::MAX, usize::MAX));
                if matches!(*track[target_x][0].borrow(), Dango::BuDaWang(_)) {
                    let (_, right) = track[target_x].split_at_mut(1);
                    right.shuffle(rng);
                } else {
                    track[target_x].shuffle(rng);
                }
                target_y = track[target_x]
                    .iter()
                    .position(|d| d.borrow().get_pos().0 == usize::MAX)
                    .expect("The dango always can be found after shuffled by the hole.");
            }
        }

        self.set_extra(0);

        self.set_pos((target_x, target_y));
        track[target_x]
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != target_y)
            .for_each(|(idx, dango)| dango.borrow_mut().set_pos((target_x, idx)));

        // drop(self);
        // for (idx, dango) in track[target_x].iter().enumerate() {
        //     dango.borrow_mut().set_pos((target_x, idx));
        // }

        target_x == (track.len() - 1)
    }

    fn before_run(&mut self, _track: &mut Track) {}

    fn after_run(&mut self, _track: &mut Track) {}
}

#[derive(Debug, Clone)]
pub struct Denia {
    last_dice: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
}

impl Denia {
    pub fn new() -> Self {
        Self {
            last_dice: 0,
            pos: (0, 0),
            extra: 0,
        }
    }
}

impl Run for Denia {
    fn get_extra(&self) -> isize {
        self.extra
    }

    fn set_extra(&mut self, extra: isize) {
        self.extra = extra
    }

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos;
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);
        if n == self.last_dice {
            self.extra += 2;
        }
        self.last_dice = n;

        self.make_step(n, track, map, rng)
    }
}

#[derive(Debug, Clone)]
pub struct Sigrika {
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
}

impl Sigrika {
    pub fn new() -> Self {
        Self {
            pos: (0, 0),
            extra: 0,
        }
    }
}

impl Run for Sigrika {
    fn get_extra(&self) -> isize {
        self.extra
    }

    fn set_extra(&mut self, extra: isize) {
        self.extra = extra
    }

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);
        self.make_step(n, track, map, rng)
    }

    fn before_run(&mut self, track: &mut Track) {
        let (x, y) = self.get_pos();
        track[x..]
            .iter()
            .enumerate()
            .flat_map(|(i, point)| {
                if i == 0 {
                    point[y + 1..].iter()
                } else {
                    point.iter()
                }
            })
            .filter(|dango| !is_budawang(dango))
            .take(2)
            .for_each(|dango| {
                let mut dango = dango.borrow_mut();
                let target_extra = dango.get_extra() - 1;
                dango.set_extra(target_extra);
            });
    }
}

#[derive(Debug, Clone)]
pub struct Hiyuki {
    /// 是否遇到过布大王
    meeted: bool,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
}

impl Hiyuki {
    pub fn new() -> Self {
        Self {
            meeted: false,
            pos: (0, 0),
            extra: 0,
        }
    }
}

impl Run for Hiyuki {
    fn get_extra(&self) -> isize {
        self.extra
    }

    fn set_extra(&mut self, extra: isize) {
        self.extra = extra
    }

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);

        let (old_x, _) = self.get_pos();

        // 当前回合绯雪比布大王后动，且被布大王经过
        if !self.meeted && is_budawang(&track[old_x][0]) {
            self.meeted = true;
        }

        self.extra = self.meeted as isize;

        let arrived = self.make_step(n, track, map, rng);

        // 移动结束后，
        if !arrived && !self.meeted {
            let (new_x, _) = self.get_pos();
            self.meeted = has_budawang(&track[old_x + 1..=new_x]);
        }

        arrived
    }

    // TODO 团子的一些状态属性，在下半场开始时需要进行重置
}

#[derive(Debug, Clone)]
pub struct Cartethyia {
    /// 是否成为过最后一名
    has_been_last: bool,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
}

impl Cartethyia {
    const EXTRA_ADVANCE_PROB: f64 = 0.6;

    pub fn new() -> Self {
        Self {
            has_been_last: false,
            pos: (0, 0),
            extra: 0,
        }
    }
}

impl Run for Cartethyia {
    fn get_extra(&self) -> isize {
        self.extra
    }

    fn set_extra(&mut self, extra: isize) {
        self.extra = extra
    }

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);
        if self.has_been_last && rng.random_bool(Cartethyia::EXTRA_ADVANCE_PROB) {
            self.extra += 2;
        }

        let arrived = self.make_step(n, track, map, rng);

        if !arrived && !self.has_been_last {
            let (x, y) = self.get_pos();
            if y == 0 && !no_dango(&track[0..x]) {
                self.has_been_last = true;
            }
        }

        arrived
    }

    // TODO 卡提希娅如果在上半场结束时是最后一名
    // 那么下半场开始时（即卡提希娅移动之前）是否触发技能
}

#[derive(Debug, Clone)]
pub struct Phoebe {
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
}

impl Phoebe {
    const EXTRA_ADVANCE_PROB: f64 = 0.5;

    pub fn new() -> Self {
        Self {
            pos: (0, 0),
            extra: 0,
        }
    }
}

impl Run for Phoebe {
    fn get_extra(&self) -> isize {
        self.extra
    }

    fn set_extra(&mut self, extra: isize) {
        self.extra = extra
    }

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);

        if rng.random_bool(Phoebe::EXTRA_ADVANCE_PROB) {
            self.extra += 1;
        }

        self.make_step(n, track, map, rng)
    }
}

#[derive(Debug, Clone)]
pub struct LuukHerssen {
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
}

impl LuukHerssen {
    pub fn new() -> Self {
        Self {
            pos: (0, 0),
            extra: 0,
        }
    }
}

impl Run for LuukHerssen {
    fn get_extra(&self) -> isize {
        self.extra
    }

    fn set_extra(&mut self, extra: isize) {
        self.extra = extra
    }

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos
    }

    fn accelerate_step(&self) -> usize {
        4
    }

    fn decelerate_step(&self) -> usize {
        2
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);
        self.make_step(n, track, map, rng)
    }
}

#[derive(Debug, Clone)]
pub struct BuDaWang {
    pos: (usize, usize),
}

impl BuDaWang {
    const BUDAWANG_DICE: [usize; 6] = [1, 2, 3, 4, 5, 6];

    pub fn new() -> Self {
        Self {
            pos: (TRACK_LEN - 1, 0),
        }
    }
}

impl Run for BuDaWang {
    fn roll<R>(&self, rng: &mut R) -> usize
    where
        R: Rng + ?Sized,
    {
        *BuDaWang::BUDAWANG_DICE.choose(rng).unwrap()
    }

    fn get_extra(&self) -> isize {
        0
    }

    fn set_extra(&mut self, _extra: isize) {}

    fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    fn set_pos(&mut self, pos: (usize, usize)) {
        self.pos = pos
    }

    fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.roll(rng);

        let (x, _) = self.get_pos(); // y==0 恒成立

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

        false
    }

    fn after_run(&mut self, track: &mut Track) {
        let (x, _) = self.get_pos();
        if track[x].len() == 1 && no_dango(&track[0..x]) {
            let self_dango = track[x].pop().unwrap();
            track[TRACK_LEN - 1].push(self_dango);
        }
    }
}

fn is_budawang(dango: &RefDango) -> bool {
    matches!(*dango.borrow(), Dango::BuDaWang(_))
}

fn has_budawang(range: &[Point]) -> bool {
    range
        .iter()
        .filter_map(|point| {
            if point.is_empty() {
                None
            } else {
                Some(&point[0])
            }
        })
        .any(is_budawang)
}

fn no_dango(range: &[Point]) -> bool {
    range.iter().rev().all(|point| {
        point.is_empty() // 团子所在位置的后面都没有其他团子
        || (point.len() == 1 && is_budawang(&point[0])) // 或者只有布大王
    })
}

#[derive(Debug, Clone, Delegate)]
#[delegate(Run)]
pub enum Dango {
    Denia(Denia),
    Sigrika(Sigrika),
    Hiyuki(Hiyuki),
    Cartethyia(Cartethyia),
    Phoebe(Phoebe),
    LuukHerssen(LuukHerssen),
    BuDaWang(BuDaWang),
}

pub type RefDango = Rc<RefCell<Dango>>;

impl Dango {
    fn new(dango: Dango) -> RefDango {
        Rc::new(RefCell::new(dango))
    }
}

pub fn new_denia() -> RefDango {
    Dango::new(Dango::Denia(Denia::new()))
}

pub fn new_sigrika() -> RefDango {
    Dango::new(Dango::Sigrika(Sigrika::new()))
}

pub fn new_hiyuki() -> RefDango {
    Dango::new(Dango::Hiyuki(Hiyuki::new()))
}

pub fn new_cartethyia() -> RefDango {
    Dango::new(Dango::Cartethyia(Cartethyia::new()))
}

pub fn new_phoebe() -> RefDango {
    Dango::new(Dango::Phoebe(Phoebe::new()))
}

pub fn new_luuk_herssen() -> RefDango {
    Dango::new(Dango::LuukHerssen(LuukHerssen::new()))
}

pub fn new_bu_da_wang() -> RefDango {
    Dango::new(Dango::BuDaWang(BuDaWang::new()))
}

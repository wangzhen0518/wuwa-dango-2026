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

macro_rules! impl_run_attrs {
    () => {
        fn get_n(&self) -> usize {
            self.n
        }

        fn set_n(&mut self, n: usize) {
            self.n = n
        }

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

        fn get_arrive_count(&self) -> usize {
            self.arrive_count
        }

        fn increase_arrive_count(&mut self) {
            self.arrive_count += 1;
        }

        fn get_target_arrive_count(&self) -> usize {
            self.target_arrive_count
        }

        fn increase_target_arrive_count(&mut self) {
            self.target_arrive_count += 1;
        }
    };
}

#[delegatable_trait]
pub trait Run {
    fn reset(&mut self) {
        self.set_extra(0);
        self.set_n(0);
    }

    fn roll<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let n = *COMMON_DICE.choose(rng).expect("Roll failed");
        self.set_n(n);
    }

    fn get_n(&self) -> usize;
    fn set_n(&mut self, n: usize);

    fn get_extra(&self) -> isize;
    fn set_extra(&mut self, extra: isize);

    fn get_pos(&self) -> (usize, usize);
    fn set_pos(&mut self, pos: (usize, usize));

    fn get_arrive_count(&self) -> usize;
    fn increase_arrive_count(&mut self);

    fn get_target_arrive_count(&self) -> usize;
    fn increase_target_arrive_count(&mut self);

    fn accelerate_step(&self) -> usize {
        1
    }

    fn decelerate_step(&self) -> usize {
        1
    }

    fn step<R>(&mut self, _dangos: &[RefDango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        self.make_step(track, map, rng)
    }

    fn make_step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.get_n().saturating_add_signed(self.get_extra()).max(1);
        let (x, y) = self.get_pos();

        // 还需要达到终点的次数
        let remain_arrive_count = self.get_target_arrive_count() - self.get_arrive_count();

        // 移除尾部元素
        let mut tail = track[x].split_off(y);

        // 计算目标行；剩余 > 1 时不限制不超过终点
        let mut target_x = if remain_arrive_count > 1 {
            x + n
        } else {
            (x + n).min(track.len() - 1)
        };

        // 越过终点
        if target_x >= track.len() {
            self.increase_arrive_count();
            tail[1..]
                .iter()
                .for_each(|dango| dango.borrow_mut().increase_arrive_count());
            target_x %= track.len();
        }

        // 将尾部元素追加到目标行
        let mut target_y = track[target_x].len();
        track[target_x].append(&mut tail);

        match map[target_x] {
            PointType::Common => {}
            PointType::Accelerate => {
                let acc = self.accelerate_step();
                let mut new_x = target_x + acc;
                if new_x >= track.len() {
                    self.increase_arrive_count();
                    track[target_x][target_y + 1..]
                        .iter()
                        .for_each(|dango| dango.borrow_mut().increase_arrive_count());
                    new_x %= track.len();
                }
                if new_x > target_x {
                    let (left, right) = track.split_at_mut(new_x);
                    target_y += right[0].len();
                    right[0].append(&mut left[target_x]);
                } else if new_x < target_x {
                    let (left, right) = track.split_at_mut(target_x);
                    target_y += left[new_x].len();
                    left[new_x].append(&mut right[0]);
                }
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
                if track[target_x].len() > 1 {
                    if is_budawang(&track[target_x][0]) {
                        let (_, right) = track[target_x].split_at_mut(1);
                        right.shuffle(rng);
                    } else {
                        track[target_x].shuffle(rng);
                    }
                    target_y = track[target_x]
                        .iter()
                        // 当前 self 已经发生了 borrow_mut，那么 try_borrow 一定会报错，而其他 dango 处不会报错
                        .position(|d| d.try_borrow().is_err())
                        .expect("The dango always can be found after shuffled by the hole.");
                }
            }
        }

        self.set_extra(0);
        self.set_pos((target_x, target_y));

        // 更新被 self 携带的团子的 pos，由于可能经过 hole 重置了顺序，所以更新所有在 target_x 处的团子的 pos
        track[target_x]
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != target_y)
            .for_each(|(idx, dango)| dango.borrow_mut().set_pos((target_x, idx)));

        self.get_arrive_count() == self.get_target_arrive_count() - 1 && target_x == track.len() - 1
    }

    fn before_run(&mut self, _dangos: &[RefDango], _track: &mut Track) {}

    fn after_run(&mut self, _dangos: &[RefDango], _track: &mut Track) {}
}

#[derive(Debug, Clone)]
pub struct Denia {
    n: usize,
    last_dice: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Denia {
    pub fn new() -> Self {
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

impl Run for Denia {
    fn reset(&mut self) {
        self.last_dice = 0;
        self.extra = 0;
        self.n = 0;
    }

    impl_run_attrs!();

    fn step<R>(&mut self, _dangos: &[RefDango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let n = self.get_n();
        if n == self.last_dice {
            self.extra += 2;
        }
        self.last_dice = n;

        self.make_step(track, map, rng)
    }
}

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

impl Run for Sigrika {
    impl_run_attrs!();

    fn before_run(&mut self, dangos: &[RefDango], track: &mut Track) {
        // 收集除布大王外，领先于自己的团子
        let mut dangos: Vec<_> = dangos
            .iter()
            .filter(|dango| {
                dango.try_borrow_mut().is_ok_and(|d| {
                    !matches!(*d, Dango::BuDaWang(_))
                        && d.get_arrive_count()
                            .cmp(&self.get_arrive_count())
                            .then(d.get_pos().cmp(&self.get_pos()))
                            .is_gt()
                })
            })
            .cloned()
            .collect();

        if !dangos.is_empty() {
            sort_dangos(&mut dangos);
            dangos.iter().rev().take(2).for_each(|dango| {
                let mut dango = dango.borrow_mut();
                let target_extra = dango.get_extra() - 1;
                dango.set_extra(target_extra);
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hiyuki {
    n: usize,
    /// 是否遇到过布大王
    meeted: bool,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Hiyuki {
    pub fn new() -> Self {
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

impl Run for Hiyuki {
    fn reset(&mut self) {
        self.meeted = false;
        self.extra = 0;
        self.n = 0;
    }

    impl_run_attrs!();

    fn step<R>(&mut self, dangos: &[RefDango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        let (old_x, _) = self.get_pos();

        // 绯雪上轮行动后至此轮行动前，被布大王经过
        if !self.meeted && is_budawang(&track[old_x][0]) {
            self.meeted = true;
        }

        self.extra = self.meeted as isize;

        let arrived = self.make_step(track, map, rng);

        // 移动结束后，
        if !arrived && !self.meeted {
            let (new_x, _) = self.get_pos();
            // 没有考虑绯雪越过终点绕一圈之后，仍 new_x > old_x 的情况，应该不会发生这种情况
            if new_x > old_x {
                self.meeted = has_budawang(&track[old_x + 1..=new_x]);
            } else {
                self.meeted = has_budawang(&track[old_x + 1..]) || has_budawang(&track[..=new_x]);
            }
        }

        arrived
    }
}

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
    const EXTRA_ADVANCE_PROB: f64 = 0.6;

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

    fn is_last(&self, dangos: &[RefDango]) -> bool {
        // 收集除自己和布大王以外、落后于自己的团子
        let mut after_self_dangos: Vec<_> = dangos
            .iter()
            .filter(|dango| {
                dango.try_borrow().is_ok_and(|d| {
                    !matches!(*d, Dango::BuDaWang(_))
                        && d.get_arrive_count()
                            .cmp(&self.get_arrive_count())
                            .then(d.get_pos().cmp(&self.get_pos()))
                            .is_lt()
                })
            })
            .cloned()
            .collect();

        if cfg!(debug_assertions) {
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

impl Run for Cartethyia {
    fn reset(&mut self) {
        self.has_been_last = false;
        self.extra = 0;
        self.n = 0;
    }

    impl_run_attrs!();

    fn step<R>(&mut self, dangos: &[RefDango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        if self.has_been_last && rng.random_bool(Cartethyia::EXTRA_ADVANCE_PROB) {
            self.extra += 2;
        }

        let arrived = self.make_step(track, map, rng);

        if !arrived && !self.has_been_last {
            if cfg!(debug_assertions) {
                if self.is_last(dangos) {
                    self.has_been_last = true
                }
            } else {
                self.has_been_last = self.is_last(dangos);
            }
        }

        arrived
    }
}

#[derive(Debug, Clone)]
pub struct Phoebe {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl Phoebe {
    const EXTRA_ADVANCE_PROB: f64 = 0.5;

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

impl Run for Phoebe {
    impl_run_attrs!();

    fn step<R>(&mut self, _dangos: &[RefDango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        if rng.random_bool(Phoebe::EXTRA_ADVANCE_PROB) {
            self.extra += 1;
        }

        self.make_step(track, map, rng)
    }
}

#[derive(Debug, Clone)]
pub struct LuukHerssen {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
}

impl LuukHerssen {
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

impl Run for LuukHerssen {
    impl_run_attrs!();

    fn accelerate_step(&self) -> usize {
        4
    }

    fn decelerate_step(&self) -> usize {
        2
    }
}

#[derive(Debug, Clone)]
pub struct BuDaWang {
    n: usize,
    pos: (usize, usize),
}

impl BuDaWang {
    const BUDAWANG_DICE: [usize; 6] = [1, 2, 3, 4, 5, 6];

    pub fn new() -> Self {
        Self {
            n: 0,
            pos: (TRACK_LEN - 1, 0),
        }
    }

    fn leave_last_dango(&self, dangos: &[RefDango]) -> bool {
        let (x, _) = self.get_pos();

        // 考虑能否提取 get_last_dango 函数
        // 收集除布大王以外的团子
        let mut dangos: Vec<_> = dangos
            .iter()
            .filter(|dango| dango.try_borrow_mut().is_ok())
            .cloned()
            .collect();

        sort_dangos(&mut dangos);

        let last_dango = dangos.last().expect("Always can get dango");
        let (last_x, last_y) = last_dango.borrow().get_pos();

        last_x > x
    }
}

impl Run for BuDaWang {
    fn roll<R>(&mut self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let n = *BuDaWang::BUDAWANG_DICE.choose(rng).expect("Roll failed");
        self.n = n;
    }

    fn get_n(&self) -> usize {
        self.n
    }

    fn set_n(&mut self, n: usize) {
        self.n = n;
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

    fn get_arrive_count(&self) -> usize {
        0
    }

    fn increase_arrive_count(&mut self) {}

    fn get_target_arrive_count(&self) -> usize {
        0
    }

    fn increase_target_arrive_count(&mut self) {}

    fn step<R>(&mut self, _dangos: &[RefDango], track: &mut Track, map: &Map, rng: &mut R) -> bool
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
            .iter()
            .enumerate()
            .skip(1)
            .for_each(|(idx, dango)| dango.borrow_mut().set_pos((target_x, idx)));

        false
    }

    fn after_run(&mut self, dangos: &[RefDango], track: &mut Track) {
        if self.leave_last_dango(dangos) {
            let (x, _) = self.get_pos();

            // remove(0) 而不是 pop，因为可能已经与最后一名分离了，但是可能有团子已经跑了一圈来到布大王上方了
            let self_dango = track[x].remove(0);
            track[x]
                .iter()
                .enumerate()
                .for_each(|(idx, dango)| dango.borrow_mut().set_pos((x, idx)));

            self.set_pos((TRACK_LEN - 1, 0));
            track[TRACK_LEN - 1].insert(0, self_dango);
            track[TRACK_LEN - 1]
                .iter()
                .enumerate()
                .skip(1)
                .for_each(|(idx, dango)| dango.borrow_mut().set_pos((TRACK_LEN - 1, idx)));
        }
    }
}

// #[inline]
// fn point_to_self<T>(pointer: &RefCell<T>, x: &T) -> bool {
//     std::ptr::eq(pointer.as_ptr(), x)
// }

pub fn is_budawang(dango: &RefDango) -> bool {
    // dango 可能是当前正在行动的团子，那么就已经发生过 borrow_mut，导致再次 borrow 失败
    // 只要 budawang 不调用这个函数，那么 try_borrow 的判断逻辑就没问题
    dango
        .try_borrow()
        .is_ok_and(|x| matches!(*x, Dango::BuDaWang(_)))

    // matches!(*dango.borrow(), Dango::BuDaWang(_))
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

pub fn sort_dangos(dangos: &mut [RefDango]) {
    dangos.sort_by(|a, b| {
        let a = a.borrow();
        let b = b.borrow();

        a.get_arrive_count()
            .cmp(&b.get_arrive_count())
            .then(a.get_pos().cmp(&b.get_pos()))
    });
    dangos.reverse();
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

    pub fn fullname(&self) -> &'static str {
        match self {
            Dango::Denia(_) => "达妮娅",
            Dango::Sigrika(_) => "西格莉卡",
            Dango::Hiyuki(_) => "绯雪",
            Dango::Cartethyia(_) => "卡提希娅",
            Dango::Phoebe(_) => "菲比",
            Dango::LuukHerssen(_) => "陆·赫斯",
            Dango::BuDaWang(_) => "布大王",
        }
    }

    pub fn shortname(&self) -> &'static str {
        match self {
            Dango::Denia(_) => "达",
            Dango::Sigrika(_) => "西",
            Dango::Hiyuki(_) => "绯",
            Dango::Cartethyia(_) => "卡",
            Dango::Phoebe(_) => "菲",
            Dango::LuukHerssen(_) => "陆",
            Dango::BuDaWang(_) => "布",
        }
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

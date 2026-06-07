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

mod budawang;
mod cartethyia;
mod denia;
mod hiyuki;
mod luukherssen;
mod phoebe;
mod sigrika;

use budawang::BuDaWang;
use cartethyia::Cartethyia;
use denia::Denia;
use hiyuki::Hiyuki;
use luukherssen::LuukHerssen;
use phoebe::Phoebe;
use sigrika::Sigrika;

static COMMON_DICE: [usize; 6] = [1, 1, 2, 2, 3, 3];

#[delegatable_trait]
pub trait Run {
    fn reset(&self) {
        self.set_extra(0);
        self.set_n(0);
    }

    fn roll<R>(&self, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let n = *COMMON_DICE.choose(rng).expect("Roll failed");
        self.set_n(n);
    }

    fn get_n(&self) -> usize;
    fn set_n(&self, n: usize);

    fn get_extra(&self) -> isize;
    fn set_extra(&self, extra: isize);

    fn get_pos(&self) -> (usize, usize);
    fn set_pos(&self, pos: (usize, usize));

    fn get_arrive_count(&self) -> usize;
    fn increase_arrive_count(&self);

    fn get_target_arrive_count(&self) -> usize;
    fn increase_target_arrive_count(&self);

    fn accelerate_step(&self) -> usize {
        1
    }

    fn decelerate_step(&self) -> usize {
        1
    }

    fn step<R>(&self, _dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
    where
        R: Rng + ?Sized,
    {
        self.make_step(track, map, rng)
    }

    fn make_step<R>(&self, track: &mut Track, map: &Map, rng: &mut R) -> bool
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
                .iter_mut()
                .for_each(|dango| dango.increase_arrive_count());
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
                    let (left, _) = track.split_at_mut(target_x + 1);
                    // target_x 处的 tail 部分（从 target_y + 1 开始）在 append 后位于 left[target_x][target_y + 1..]
                    left[target_x][target_y + 1..]
                        .iter_mut()
                        .for_each(|dango| dango.increase_arrive_count());
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
                }
            }
        }

        self.set_extra(0);

        // 更新被 self 携带的团子的 pos，由于可能经过 hole 重置了顺序，所以更新所有在 target_x 处的团子的 pos
        track[target_x]
            .iter_mut()
            .enumerate()
            .for_each(|(idx, dango)| dango.set_pos((target_x, idx)));

        self.get_arrive_count() == self.get_target_arrive_count() - 1 && target_x == track.len() - 1
    }

    fn before_run(&self, _dangos: &[Dango], _track: &mut Track) {}

    fn after_run(&self, _dangos: &[Dango], _track: &mut Track) {}
}

macro_rules! impl_run_helper {
    () => {
        fn get_n(&self) -> usize {
            self.borrow().n
        }

        fn set_n(&self, n: usize) {
            self.borrow_mut().n = n;
        }

        fn get_extra(&self) -> isize {
            self.borrow().extra
        }

        fn set_extra(&self, extra: isize) {
            self.borrow_mut().extra = extra
        }

        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }

        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos
        }

        fn get_arrive_count(&self) -> usize {
            self.borrow().arrive_count
        }

        fn increase_arrive_count(&self) {
            self.borrow_mut().arrive_count += 1;
        }

        fn get_target_arrive_count(&self) -> usize {
            self.borrow().target_arrive_count
        }

        fn increase_target_arrive_count(&self) {
            self.borrow_mut().target_arrive_count += 1;
        }
    };
}

pub fn is_budawang(dango: &Dango) -> bool {
    matches!(dango, Dango::BuDaWang(_))
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

pub fn sort_dangos(dangos: &mut [Dango]) {
    dangos.sort_by(|a, b| {
        a.get_arrive_count()
            .cmp(&b.get_arrive_count())
            .then(a.get_pos().cmp(&b.get_pos()))
    });
    dangos.reverse();
}

#[derive(Debug, Clone)]
// #[delegate(Run)]
pub enum Dango {
    Denia(Rc<RefCell<Denia>>),
    Sigrika(Rc<RefCell<Sigrika>>),
    Hiyuki(Rc<RefCell<Hiyuki>>),
    Cartethyia(Rc<RefCell<Cartethyia>>),
    Phoebe(Rc<RefCell<Phoebe>>),
    LuukHerssen(Rc<RefCell<LuukHerssen>>),
    BuDaWang(Rc<RefCell<BuDaWang>>),
}

impl Dango {
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

macro_rules! impl_run_for_dango_helper {
    (
        $(
            $name:ident
            $(<$($gen:tt),*>)?
            (
                &self
                $(, $arg:ident : $arg_ty:ty )*
            )
            $(-> $ret:ty)?
            $( [ where $($where:tt)* ])?
            ;
        )*
    ) => {
        $(
            fn $name $(<$($gen),*>)? (
                &self
                $(, $arg : $arg_ty )*
            )
            $(-> $ret)?
            $(where $($where)*)?
            {
                match self {
                    Dango::Denia(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Sigrika(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Hiyuki(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Cartethyia(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Phoebe(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::LuukHerssen(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::BuDaWang(ref_cell) => ref_cell.$name($($arg),*),
                }
            }
        )*
    };
}

impl Run for Dango {
    impl_run_for_dango_helper!(
        reset(&self);
        roll<R>(&self, rng: &mut R) [where R: Rng + ?Sized];
        get_n(&self) -> usize;
        set_n(&self, n: usize);

        get_extra(&self) -> isize;
        set_extra(&self, extra: isize);

        get_pos(&self) -> (usize, usize);
        set_pos(&self, pos: (usize, usize));

        get_arrive_count(&self) -> usize;
        increase_arrive_count(&self);

        get_target_arrive_count(&self) -> usize;
        increase_target_arrive_count(&self);

        accelerate_step(&self) -> usize ;

        decelerate_step(&self) -> usize ;

        step<R>(&self, dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool [where R: Rng + ?Sized];

        make_step<R>(&self, track: &mut Track, map: &Map, rng: &mut R) -> bool [where R: Rng + ?Sized];

        before_run(&self, dangos: &[Dango], track: &mut Track);
        after_run(&self, dangos: &[Dango], track: &mut Track);
    );
}

pub fn new_denia() -> Dango {
    Dango::Denia(Rc::new(RefCell::new(Denia::new())))
}

pub fn new_sigrika() -> Dango {
    Dango::Sigrika(Rc::new(RefCell::new(Sigrika::new())))
}

pub fn new_hiyuki() -> Dango {
    Dango::Hiyuki(Rc::new(RefCell::new(Hiyuki::new())))
}

pub fn new_cartethyia() -> Dango {
    Dango::Cartethyia(Rc::new(RefCell::new(Cartethyia::new())))
}

pub fn new_phoebe() -> Dango {
    Dango::Phoebe(Rc::new(RefCell::new(Phoebe::new())))
}

pub fn new_luuk_herssen() -> Dango {
    Dango::LuukHerssen(Rc::new(RefCell::new(LuukHerssen::new())))
}

pub fn new_bu_da_wang() -> Dango {
    Dango::BuDaWang(Rc::new(RefCell::new(BuDaWang::new())))
}

pub(in crate::dangos) use {impl_run_for_dango_helper, impl_run_helper};

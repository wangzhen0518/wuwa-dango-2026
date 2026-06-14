#[allow(unused_imports)]
use std::{cell::RefCell, fmt::Write, rc::Rc};

// use ambassador::delegatable_trait;
use rand::{
    Rng,
    seq::{IndexedRandom, SliceRandom},
};

use crate::track::{Map, Point, PointType, Track};

pub mod budawang;
pub mod cartethyia;
pub mod denia;
pub mod hiyuki;
pub mod luukherssen;
pub mod phoebe;
pub mod sigrika;

use budawang::BuDaWang;
use cartethyia::Cartethyia;
use denia::Denia;
use hiyuki::Hiyuki;
use luukherssen::LuukHerssen;
use phoebe::Phoebe;
use sigrika::Sigrika;

static COMMON_DICE: [usize; 6] = [1, 1, 2, 2, 3, 3];

// #[delegatable_trait]
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

    #[allow(unused_variables)]
    fn step<R>(&self, dangos: &[Dango], track: &mut Track, map: &Map, rng: &mut R) -> bool
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
            tail[1..] //TODO 为什么从 1 开始索引？
                .iter_mut()
                .for_each(|dango| dango.increase_arrive_count());
            target_x %= track.len();
        }

        // 将尾部元素追加到目标行
        #[allow(unused_mut)]
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
                    // target_y += right[0].len();
                    right[0].append(&mut left[target_x]);
                } else if new_x < target_x {
                    let (left, right) = track.split_at_mut(target_x);
                    // target_y += left[new_x].len();
                    left[new_x].append(&mut right[0]);
                }
                target_x = new_x;
            }
            PointType::Decelerate => {
                let dec = self.decelerate_step();
                let new_x = target_x.saturating_sub(dec);
                let (left, right) = track.split_at_mut(target_x);

                // target_y += left[new_x].len();
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

    #[allow(unused_variables)]
    fn before_run(&self, dangos: &[Dango], track: &mut Track) {}

    #[allow(unused_variables)]
    fn after_run(&self, dangos: &[Dango], track: &mut Track) {}
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

pub(in crate::dangos) use impl_run_helper;

#[derive(Debug, Clone)]
// #[delegate(Run)]
pub enum Dango {
    BuDaWang(Rc<RefCell<BuDaWang>>),
    Cartethyia(Rc<RefCell<Cartethyia>>),
    Denia(Rc<RefCell<Denia>>),
    Hiyuki(Rc<RefCell<Hiyuki>>),
    LuukHerssen(Rc<RefCell<LuukHerssen>>),
    Phoebe(Rc<RefCell<Phoebe>>),
    Sigrika(Rc<RefCell<Sigrika>>),
}

impl Dango {
    pub fn default_budawang() -> Dango {
        Dango::BuDaWang(budawang::default_budawang())
    }

    pub fn default_cartethyia() -> Dango {
        Dango::Cartethyia(cartethyia::default_cartethyia())
    }

    pub fn default_denia() -> Dango {
        Dango::Denia(denia::default_denia())
    }

    pub fn default_hiyuki() -> Dango {
        Dango::Hiyuki(hiyuki::default_hiyuki())
    }

    pub fn default_luukherssen() -> Dango {
        Dango::LuukHerssen(luukherssen::default_luukherssen())
    }

    pub fn default_phoebe() -> Dango {
        Dango::Phoebe(phoebe::default_phoebe())
    }

    pub fn default_sigrika() -> Dango {
        Dango::Sigrika(sigrika::default_sigrika())
    }

    #[allow(dead_code)]
    pub fn new_budawang(n: usize, pos: (usize, usize)) -> Dango {
        Dango::BuDaWang(budawang::new_budawang(n, pos))
    }

    #[allow(dead_code)]
    pub fn new_cartethyia(
        n: usize,
        pos: (usize, usize),
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
        has_been_last: bool,
    ) -> Dango {
        Dango::Cartethyia(cartethyia::new_cartethyia(
            n,
            pos,
            extra,
            arrive_count,
            target_arrive_count,
            has_been_last,
        ))
    }

    #[allow(dead_code)]
    pub fn new_denia(
        n: usize,
        pos: (usize, usize),
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
        last_dice: usize,
    ) -> Dango {
        Dango::Denia(denia::new_denia(
            n,
            pos,
            extra,
            arrive_count,
            target_arrive_count,
            last_dice,
        ))
    }

    #[allow(dead_code)]
    pub fn new_hiyuki(
        n: usize,
        pos: (usize, usize),
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
        meeted: bool,
    ) -> Dango {
        Dango::Hiyuki(hiyuki::new_hiyuki(
            n,
            pos,
            extra,
            arrive_count,
            target_arrive_count,
            meeted,
        ))
    }

    #[allow(dead_code)]
    pub fn new_luukherssen(
        n: usize,
        pos: (usize, usize),
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    ) -> Dango {
        Dango::LuukHerssen(luukherssen::new_luukherssen(
            n,
            pos,
            extra,
            arrive_count,
            target_arrive_count,
        ))
    }

    #[allow(dead_code)]
    pub fn new_phoebe(
        n: usize,
        pos: (usize, usize),
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    ) -> Dango {
        Dango::Phoebe(phoebe::new_phoebe(
            n,
            pos,
            extra,
            arrive_count,
            target_arrive_count,
        ))
    }

    #[allow(dead_code)]
    pub fn new_sigrika(
        n: usize,
        pos: (usize, usize),
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    ) -> Dango {
        Dango::Sigrika(sigrika::new_sigrika(
            n,
            pos,
            extra,
            arrive_count,
            target_arrive_count,
        ))
    }
}

macro_rules! from_variant_helper {
    ($($ty:ident),*) => {
        $(
            impl From<Rc<RefCell<$ty>>> for Dango {
                fn from(value: Rc<RefCell<$ty>>) -> Dango {
                    Dango::$ty(value)
                }
            }
        )*
    };
}

from_variant_helper!(
    BuDaWang,
    Cartethyia,
    Denia,
    Hiyuki,
    LuukHerssen,
    Phoebe,
    Sigrika
);

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
                    Dango::BuDaWang(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Cartethyia(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Denia(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Hiyuki(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::LuukHerssen(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Phoebe(ref_cell) => ref_cell.$name($($arg),*),
                    Dango::Sigrika(ref_cell) => ref_cell.$name($($arg),*),
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

impl PartialEq for Dango {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
            && self.get_arrive_count() == other.get_arrive_count()
            && self.get_pos() == other.get_pos()
    }
}

impl Eq for Dango {}

impl PartialOrd for Dango {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Dango {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_arrive_count()
            .cmp(&other.get_arrive_count())
            .then(self.get_pos().cmp(&other.get_pos()))
    }
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

// fn no_dango(range: &[Point]) -> bool {
//     range.iter().rev().all(|point| {
//         point.is_empty() // 团子所在位置的后面都没有其他团子
//         || (point.len() == 1 && is_budawang(&point[0])) // 或者只有布大王
//     })
// }

pub fn sort_dangos(dangos: &mut [Dango]) {
    dangos.sort_unstable_by(|a, b| b.cmp(a));
}

#[cfg(debug_assertions)]
#[allow(unused)]
pub fn show_dangos(dangos: &[Dango]) {
    let mut rank_info = String::with_capacity(10 * dangos.len());
    for dango in dangos.iter() {
        let (x, y) = dango.get_pos();
        write!(
            &mut rank_info,
            "{}({}, {})({}), ",
            DangoKind::from(dango).shortname(),
            x,
            y,
            dango.get_arrive_count()
        )
        .expect("Write failed");
    }
    println!("{}", rank_info);
}

#[cfg(not(debug_assertions))]
#[allow(unused)]
pub fn show_dangos(dangos: &[Dango]) {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DangoKind {
    BuDaWang,
    Cartethyia,
    Denia,
    Hiyuki,
    LuukHerssen,
    Phoebe,
    Sigrika,
}

impl DangoKind {
    #[allow(dead_code)]
    pub fn fullname(self) -> &'static str {
        match self {
            DangoKind::BuDaWang => "布大王",
            DangoKind::Cartethyia => "卡提希娅",
            DangoKind::Denia => "达妮娅",
            DangoKind::Hiyuki => "绯雪",
            DangoKind::LuukHerssen => "陆·赫斯",
            DangoKind::Phoebe => "菲比",
            DangoKind::Sigrika => "西格莉卡",
        }
    }

    pub fn shortname(self) -> &'static str {
        match self {
            DangoKind::BuDaWang => "布",
            DangoKind::Cartethyia => "卡",
            DangoKind::Denia => "达",
            DangoKind::Hiyuki => "绯",
            DangoKind::LuukHerssen => "陆",
            DangoKind::Phoebe => "菲",
            DangoKind::Sigrika => "西",
        }
    }
}

impl From<&Dango> for DangoKind {
    fn from(value: &Dango) -> Self {
        match value {
            Dango::BuDaWang(_) => DangoKind::BuDaWang,
            Dango::Cartethyia(_) => DangoKind::Cartethyia,
            Dango::Denia(_) => DangoKind::Denia,
            Dango::Hiyuki(_) => DangoKind::Hiyuki,
            Dango::LuukHerssen(_) => DangoKind::LuukHerssen,
            Dango::Phoebe(_) => DangoKind::Phoebe,
            Dango::Sigrika(_) => DangoKind::Sigrika,
        }
    }
}

#[cfg(test)]
mod tests;

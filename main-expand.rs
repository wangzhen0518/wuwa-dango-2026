#![feature(prelude_import)]
#![allow(unused)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::{fmt::Write, ops::DerefMut};
use ambassador::{Delegate, delegatable_trait_remote};
use rand::{
    Rng, SeedableRng, rngs::{StdRng, ThreadRng},
    seq::SliceRandom,
};
use crate::{
    dangos::{RefDango, Run, is_budawang},
    track::{Map, TRACK_LEN, Track, init_map, init_track, show_track},
};
mod dangos {
    use std::{cell::RefCell, rc::Rc};
    use ambassador::{Delegate, delegatable_trait};
    use rand::{Rng, RngExt, seq::{IndexedRandom, SliceRandom}};
    use crate::{
        track::{Map, Point, PointType, TRACK_LEN, Track},
        utils::split_first,
    };
    static COMMON_DICE: [usize; 6] = [1, 1, 2, 2, 3, 3];
    pub trait Run {
        fn reset(&mut self) {
            self.set_extra(0);
        }
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
        fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
        where
            R: Rng + ?Sized,
        {
            let n = self.roll(rng);
            self.make_step(n, track, map, rng)
        }
        fn make_step<R>(
            &mut self,
            n: usize,
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
        where
            R: Rng + ?Sized,
        {
            let n = n.saturating_add_signed(self.get_extra()).max(1);
            let (x, y) = self.get_pos();
            let remain_arrive_count = self.get_target_arrive_count()
                - self.get_arrive_count();
            let mut tail = track[x].split_off(y);
            let mut target_x = if remain_arrive_count > 1 {
                x + n
            } else {
                (x + n).min(track.len() - 1)
            };
            if target_x >= track.len() {
                self.increase_arrive_count();
                tail[1..]
                    .iter()
                    .for_each(|dango| dango.borrow_mut().increase_arrive_count());
                target_x %= track.len();
            }
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
                            .for_each(|dango| {
                                dango.borrow_mut().increase_arrive_count()
                            });
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
                    target_x = new_x;
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
                            .position(|d| d.try_borrow().is_err())
                            .expect(
                                "The dango always can be found after shuffled by the hole.",
                            );
                    }
                }
            }
            self.set_extra(0);
            self.set_pos((target_x, target_y));
            track[target_x]
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != target_y)
                .for_each(|(idx, dango)| dango.borrow_mut().set_pos((target_x, idx)));
            self.get_arrive_count() == self.get_target_arrive_count() - 1
                && target_x == track.len() - 1
        }
        fn before_run(&mut self, _track: &mut Track) {}
        fn after_run(&mut self, _track: &mut Track) {}
    }
    #[doc(inline)]
    ///A macro to be used by [`ambassador::Delegate`] to delegate [`Run`]
    pub use _ambassador_impl_Run as ambassador_impl_Run;
    #[doc(hidden)]
    #[allow(non_snake_case)]
    pub mod ambassador_impl_Run {}
    pub struct Denia {
        last_dice: usize,
        /// (track position, height)
        pos: (usize, usize),
        /// buff 或 debuff 效果
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Denia {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Denia",
                "last_dice",
                &self.last_dice,
                "pos",
                &self.pos,
                "extra",
                &self.extra,
                "arrive_count",
                &self.arrive_count,
                "target_arrive_count",
                &&self.target_arrive_count,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Denia {
        #[inline]
        fn clone(&self) -> Denia {
            Denia {
                last_dice: ::core::clone::Clone::clone(&self.last_dice),
                pos: ::core::clone::Clone::clone(&self.pos),
                extra: ::core::clone::Clone::clone(&self.extra),
                arrive_count: ::core::clone::Clone::clone(&self.arrive_count),
                target_arrive_count: ::core::clone::Clone::clone(
                    &self.target_arrive_count,
                ),
            }
        }
    }
    impl Denia {
        pub fn new() -> Self {
            Self {
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
        }
        fn get_extra(&self) -> isize {
            self.extra
        }
        fn set_extra(&mut self, extra: isize) {
            self.extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.pos
        }
        fn set_pos(&mut self, pos: (usize, usize)) {
            self.pos = pos;
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
    pub struct Sigrika {
        /// (track position, height)
        pos: (usize, usize),
        /// buff 或 debuff 效果
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Sigrika {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "Sigrika",
                "pos",
                &self.pos,
                "extra",
                &self.extra,
                "arrive_count",
                &self.arrive_count,
                "target_arrive_count",
                &&self.target_arrive_count,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Sigrika {
        #[inline]
        fn clone(&self) -> Sigrika {
            Sigrika {
                pos: ::core::clone::Clone::clone(&self.pos),
                extra: ::core::clone::Clone::clone(&self.extra),
                arrive_count: ::core::clone::Clone::clone(&self.arrive_count),
                target_arrive_count: ::core::clone::Clone::clone(
                    &self.target_arrive_count,
                ),
            }
        }
    }
    impl Sigrika {
        pub fn new() -> Self {
            Self {
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for Sigrika {
        fn get_extra(&self) -> isize {
            self.extra
        }
        fn set_extra(&mut self, extra: isize) {
            self.extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.pos
        }
        fn set_pos(&mut self, pos: (usize, usize)) {
            self.pos = pos;
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
        fn before_run(&mut self, track: &mut Track) {
            let (x, y) = self.get_pos();
            track[x..]
                .iter()
                .enumerate()
                .flat_map(|(i, point)| {
                    if i == 0 { point[y + 1..].iter() } else { point.iter() }
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
    pub struct Hiyuki {
        /// 是否遇到过布大王
        meeted: bool,
        /// (track position, height)
        pos: (usize, usize),
        /// buff 或 debuff 效果
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Hiyuki {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Hiyuki",
                "meeted",
                &self.meeted,
                "pos",
                &self.pos,
                "extra",
                &self.extra,
                "arrive_count",
                &self.arrive_count,
                "target_arrive_count",
                &&self.target_arrive_count,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Hiyuki {
        #[inline]
        fn clone(&self) -> Hiyuki {
            Hiyuki {
                meeted: ::core::clone::Clone::clone(&self.meeted),
                pos: ::core::clone::Clone::clone(&self.pos),
                extra: ::core::clone::Clone::clone(&self.extra),
                arrive_count: ::core::clone::Clone::clone(&self.arrive_count),
                target_arrive_count: ::core::clone::Clone::clone(
                    &self.target_arrive_count,
                ),
            }
        }
    }
    impl Hiyuki {
        pub fn new() -> Self {
            Self {
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
        }
        fn get_extra(&self) -> isize {
            self.extra
        }
        fn set_extra(&mut self, extra: isize) {
            self.extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.pos
        }
        fn set_pos(&mut self, pos: (usize, usize)) {
            self.pos = pos;
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
        fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
        where
            R: Rng + ?Sized,
        {
            let n = self.roll(rng);
            let (old_x, _) = self.get_pos();
            if !self.meeted && is_budawang(&track[old_x][0]) {
                self.meeted = true;
            }
            self.extra = self.meeted as isize;
            let arrived = self.make_step(n, track, map, rng);
            if !arrived && !self.meeted {
                let (new_x, _) = self.get_pos();
                if new_x > old_x {
                    self.meeted = has_budawang(&track[old_x + 1..=new_x]);
                } else {
                    self.meeted = has_budawang(&track[old_x + 1..])
                        || has_budawang(&track[..=new_x]);
                }
            }
            arrived
        }
    }
    pub struct Cartethyia {
        /// 是否成为过最后一名
        has_been_last: bool,
        /// (track position, height)
        pos: (usize, usize),
        /// buff 或 debuff 效果
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Cartethyia {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Cartethyia",
                "has_been_last",
                &self.has_been_last,
                "pos",
                &self.pos,
                "extra",
                &self.extra,
                "arrive_count",
                &self.arrive_count,
                "target_arrive_count",
                &&self.target_arrive_count,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Cartethyia {
        #[inline]
        fn clone(&self) -> Cartethyia {
            Cartethyia {
                has_been_last: ::core::clone::Clone::clone(&self.has_been_last),
                pos: ::core::clone::Clone::clone(&self.pos),
                extra: ::core::clone::Clone::clone(&self.extra),
                arrive_count: ::core::clone::Clone::clone(&self.arrive_count),
                target_arrive_count: ::core::clone::Clone::clone(
                    &self.target_arrive_count,
                ),
            }
        }
    }
    impl Cartethyia {
        const EXTRA_ADVANCE_PROB: f64 = 0.6;
        pub fn new() -> Self {
            Self {
                has_been_last: false,
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for Cartethyia {
        fn reset(&mut self) {
            self.has_been_last = false;
            self.extra = 0;
        }
        fn get_extra(&self) -> isize {
            self.extra
        }
        fn set_extra(&mut self, extra: isize) {
            self.extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.pos
        }
        fn set_pos(&mut self, pos: (usize, usize)) {
            self.pos = pos;
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
    }
    pub struct Phoebe {
        /// (track position, height)
        pos: (usize, usize),
        /// buff 或 debuff 效果
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Phoebe {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "Phoebe",
                "pos",
                &self.pos,
                "extra",
                &self.extra,
                "arrive_count",
                &self.arrive_count,
                "target_arrive_count",
                &&self.target_arrive_count,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Phoebe {
        #[inline]
        fn clone(&self) -> Phoebe {
            Phoebe {
                pos: ::core::clone::Clone::clone(&self.pos),
                extra: ::core::clone::Clone::clone(&self.extra),
                arrive_count: ::core::clone::Clone::clone(&self.arrive_count),
                target_arrive_count: ::core::clone::Clone::clone(
                    &self.target_arrive_count,
                ),
            }
        }
    }
    impl Phoebe {
        const EXTRA_ADVANCE_PROB: f64 = 0.5;
        pub fn new() -> Self {
            Self {
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for Phoebe {
        fn get_extra(&self) -> isize {
            self.extra
        }
        fn set_extra(&mut self, extra: isize) {
            self.extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.pos
        }
        fn set_pos(&mut self, pos: (usize, usize)) {
            self.pos = pos;
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
    pub struct LuukHerssen {
        /// (track position, height)
        pos: (usize, usize),
        /// buff 或 debuff 效果
        extra: isize,
        arrive_count: usize,
        target_arrive_count: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LuukHerssen {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "LuukHerssen",
                "pos",
                &self.pos,
                "extra",
                &self.extra,
                "arrive_count",
                &self.arrive_count,
                "target_arrive_count",
                &&self.target_arrive_count,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LuukHerssen {
        #[inline]
        fn clone(&self) -> LuukHerssen {
            LuukHerssen {
                pos: ::core::clone::Clone::clone(&self.pos),
                extra: ::core::clone::Clone::clone(&self.extra),
                arrive_count: ::core::clone::Clone::clone(&self.arrive_count),
                target_arrive_count: ::core::clone::Clone::clone(
                    &self.target_arrive_count,
                ),
            }
        }
    }
    impl LuukHerssen {
        pub fn new() -> Self {
            Self {
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for LuukHerssen {
        fn get_extra(&self) -> isize {
            self.extra
        }
        fn set_extra(&mut self, extra: isize) {
            self.extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.pos
        }
        fn set_pos(&mut self, pos: (usize, usize)) {
            self.pos = pos;
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
        fn accelerate_step(&self) -> usize {
            4
        }
        fn decelerate_step(&self) -> usize {
            2
        }
    }
    pub struct BuDaWang {
        pos: (usize, usize),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for BuDaWang {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "BuDaWang",
                "pos",
                &&self.pos,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for BuDaWang {
        #[inline]
        fn clone(&self) -> BuDaWang {
            BuDaWang {
                pos: ::core::clone::Clone::clone(&self.pos),
            }
        }
    }
    impl BuDaWang {
        const BUDAWANG_DICE: [usize; 6] = [1, 2, 3, 4, 5, 6];
        pub fn new() -> Self {
            Self { pos: (TRACK_LEN - 1, 0) }
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
        fn get_arrive_count(&self) -> usize {
            0
        }
        fn increase_arrive_count(&mut self) {}
        fn get_target_arrive_count(&self) -> usize {
            0
        }
        fn increase_target_arrive_count(&mut self) {}
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
            let (x, _) = self.get_pos();
            let mut target_x = x.saturating_sub(n);
            let (self_dango, mut tail) = split_first(std::mem::take(&mut track[x]));
            let (left, right) = track.split_at_mut(target_x + 1);
            left[target_x].insert(0, self_dango);
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
                    let (left, right) = track.split_at_mut(target_x + 1);
                    right[0].append(&mut left[target_x]);
                    target_x += 1;
                }
                PointType::Decelerate => {
                    let (self_dango, mut tail) = split_first(
                        std::mem::take(&mut track[target_x]),
                    );
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
        fn after_run(&mut self, track: &mut Track) {
            let (x, _) = self.get_pos();
            if track[x].len() == 1 && no_dango(&track[0..x]) {
                let self_dango = track[x].pop().unwrap();
                track[TRACK_LEN - 1].insert(0, self_dango);
                self.set_pos((TRACK_LEN - 1, 0));
            }
        }
    }
    pub fn is_budawang(dango: &RefDango) -> bool {
        dango
            .try_borrow()
            .is_ok_and(|x| {
                #[allow(non_exhaustive_omitted_patterns)]
                match *x {
                    Dango::BuDaWang(_) => true,
                    _ => false,
                }
            })
    }
    fn has_budawang(range: &[Point]) -> bool {
        range
            .iter()
            .filter_map(|point| {
                if point.is_empty() { None } else { Some(&point[0]) }
            })
            .any(is_budawang)
    }
    fn no_dango(range: &[Point]) -> bool {
        range
            .iter()
            .rev()
            .all(|point| {
                point.is_empty() || (point.len() == 1 && is_budawang(&point[0]))
            })
    }
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
    #[automatically_derived]
    impl ::core::fmt::Debug for Dango {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Dango::Denia(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Denia",
                        &__self_0,
                    )
                }
                Dango::Sigrika(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Sigrika",
                        &__self_0,
                    )
                }
                Dango::Hiyuki(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Hiyuki",
                        &__self_0,
                    )
                }
                Dango::Cartethyia(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Cartethyia",
                        &__self_0,
                    )
                }
                Dango::Phoebe(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Phoebe",
                        &__self_0,
                    )
                }
                Dango::LuukHerssen(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "LuukHerssen",
                        &__self_0,
                    )
                }
                Dango::BuDaWang(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "BuDaWang",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Dango {
        #[inline]
        fn clone(&self) -> Dango {
            match self {
                Dango::Denia(__self_0) => {
                    Dango::Denia(::core::clone::Clone::clone(__self_0))
                }
                Dango::Sigrika(__self_0) => {
                    Dango::Sigrika(::core::clone::Clone::clone(__self_0))
                }
                Dango::Hiyuki(__self_0) => {
                    Dango::Hiyuki(::core::clone::Clone::clone(__self_0))
                }
                Dango::Cartethyia(__self_0) => {
                    Dango::Cartethyia(::core::clone::Clone::clone(__self_0))
                }
                Dango::Phoebe(__self_0) => {
                    Dango::Phoebe(::core::clone::Clone::clone(__self_0))
                }
                Dango::LuukHerssen(__self_0) => {
                    Dango::LuukHerssen(::core::clone::Clone::clone(__self_0))
                }
                Dango::BuDaWang(__self_0) => {
                    Dango::BuDaWang(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[allow(non_snake_case)]
    mod ambassador_module_Run_for_Dango {
        use super::*;
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub trait MatchRun<ambassador_X: Run>: Run {}
        #[allow(non_camel_case_types)]
        impl<ambassador_X: Run, ambassador_Y: Run> MatchRun<ambassador_X>
        for ambassador_Y {}
        impl Run for Dango
        where
            BuDaWang: Run,
            Denia: MatchRun<BuDaWang>,
            Sigrika: MatchRun<BuDaWang>,
            Hiyuki: MatchRun<BuDaWang>,
            Cartethyia: MatchRun<BuDaWang>,
            Phoebe: MatchRun<BuDaWang>,
            LuukHerssen: MatchRun<BuDaWang>,
        {
            #[inline]
            #[allow(unused_braces)]
            fn reset(&mut self) {
                match self {
                    Dango::Denia(inner) => return Run::reset(inner),
                    Dango::Sigrika(inner) => return Run::reset(inner),
                    Dango::Hiyuki(inner) => return Run::reset(inner),
                    Dango::Cartethyia(inner) => return Run::reset(inner),
                    Dango::Phoebe(inner) => return Run::reset(inner),
                    Dango::LuukHerssen(inner) => return Run::reset(inner),
                    Dango::BuDaWang(inner) => return Run::reset(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn roll<R>(&self, rng: &mut R) -> usize
            where
                R: Rng + ?Sized,
            {
                match self {
                    Dango::Denia(inner) => return Run::roll::<R>(inner, rng),
                    Dango::Sigrika(inner) => return Run::roll::<R>(inner, rng),
                    Dango::Hiyuki(inner) => return Run::roll::<R>(inner, rng),
                    Dango::Cartethyia(inner) => return Run::roll::<R>(inner, rng),
                    Dango::Phoebe(inner) => return Run::roll::<R>(inner, rng),
                    Dango::LuukHerssen(inner) => return Run::roll::<R>(inner, rng),
                    Dango::BuDaWang(inner) => return Run::roll::<R>(inner, rng),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn get_extra(&self) -> isize {
                match self {
                    Dango::Denia(inner) => return Run::get_extra(inner),
                    Dango::Sigrika(inner) => return Run::get_extra(inner),
                    Dango::Hiyuki(inner) => return Run::get_extra(inner),
                    Dango::Cartethyia(inner) => return Run::get_extra(inner),
                    Dango::Phoebe(inner) => return Run::get_extra(inner),
                    Dango::LuukHerssen(inner) => return Run::get_extra(inner),
                    Dango::BuDaWang(inner) => return Run::get_extra(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn set_extra(&mut self, extra: isize) {
                match self {
                    Dango::Denia(inner) => return Run::set_extra(inner, extra),
                    Dango::Sigrika(inner) => return Run::set_extra(inner, extra),
                    Dango::Hiyuki(inner) => return Run::set_extra(inner, extra),
                    Dango::Cartethyia(inner) => return Run::set_extra(inner, extra),
                    Dango::Phoebe(inner) => return Run::set_extra(inner, extra),
                    Dango::LuukHerssen(inner) => return Run::set_extra(inner, extra),
                    Dango::BuDaWang(inner) => return Run::set_extra(inner, extra),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn get_pos(&self) -> (usize, usize) {
                match self {
                    Dango::Denia(inner) => return Run::get_pos(inner),
                    Dango::Sigrika(inner) => return Run::get_pos(inner),
                    Dango::Hiyuki(inner) => return Run::get_pos(inner),
                    Dango::Cartethyia(inner) => return Run::get_pos(inner),
                    Dango::Phoebe(inner) => return Run::get_pos(inner),
                    Dango::LuukHerssen(inner) => return Run::get_pos(inner),
                    Dango::BuDaWang(inner) => return Run::get_pos(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn set_pos(&mut self, pos: (usize, usize)) {
                match self {
                    Dango::Denia(inner) => return Run::set_pos(inner, pos),
                    Dango::Sigrika(inner) => return Run::set_pos(inner, pos),
                    Dango::Hiyuki(inner) => return Run::set_pos(inner, pos),
                    Dango::Cartethyia(inner) => return Run::set_pos(inner, pos),
                    Dango::Phoebe(inner) => return Run::set_pos(inner, pos),
                    Dango::LuukHerssen(inner) => return Run::set_pos(inner, pos),
                    Dango::BuDaWang(inner) => return Run::set_pos(inner, pos),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn get_arrive_count(&self) -> usize {
                match self {
                    Dango::Denia(inner) => return Run::get_arrive_count(inner),
                    Dango::Sigrika(inner) => return Run::get_arrive_count(inner),
                    Dango::Hiyuki(inner) => return Run::get_arrive_count(inner),
                    Dango::Cartethyia(inner) => return Run::get_arrive_count(inner),
                    Dango::Phoebe(inner) => return Run::get_arrive_count(inner),
                    Dango::LuukHerssen(inner) => return Run::get_arrive_count(inner),
                    Dango::BuDaWang(inner) => return Run::get_arrive_count(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn increase_arrive_count(&mut self) {
                match self {
                    Dango::Denia(inner) => return Run::increase_arrive_count(inner),
                    Dango::Sigrika(inner) => return Run::increase_arrive_count(inner),
                    Dango::Hiyuki(inner) => return Run::increase_arrive_count(inner),
                    Dango::Cartethyia(inner) => return Run::increase_arrive_count(inner),
                    Dango::Phoebe(inner) => return Run::increase_arrive_count(inner),
                    Dango::LuukHerssen(inner) => return Run::increase_arrive_count(inner),
                    Dango::BuDaWang(inner) => return Run::increase_arrive_count(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn get_target_arrive_count(&self) -> usize {
                match self {
                    Dango::Denia(inner) => return Run::get_target_arrive_count(inner),
                    Dango::Sigrika(inner) => return Run::get_target_arrive_count(inner),
                    Dango::Hiyuki(inner) => return Run::get_target_arrive_count(inner),
                    Dango::Cartethyia(inner) => {
                        return Run::get_target_arrive_count(inner);
                    }
                    Dango::Phoebe(inner) => return Run::get_target_arrive_count(inner),
                    Dango::LuukHerssen(inner) => {
                        return Run::get_target_arrive_count(inner);
                    }
                    Dango::BuDaWang(inner) => return Run::get_target_arrive_count(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn increase_target_arrive_count(&mut self) {
                match self {
                    Dango::Denia(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                    Dango::Sigrika(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                    Dango::Hiyuki(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                    Dango::Cartethyia(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                    Dango::Phoebe(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                    Dango::LuukHerssen(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                    Dango::BuDaWang(inner) => {
                        return Run::increase_target_arrive_count(inner);
                    }
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn accelerate_step(&self) -> usize {
                match self {
                    Dango::Denia(inner) => return Run::accelerate_step(inner),
                    Dango::Sigrika(inner) => return Run::accelerate_step(inner),
                    Dango::Hiyuki(inner) => return Run::accelerate_step(inner),
                    Dango::Cartethyia(inner) => return Run::accelerate_step(inner),
                    Dango::Phoebe(inner) => return Run::accelerate_step(inner),
                    Dango::LuukHerssen(inner) => return Run::accelerate_step(inner),
                    Dango::BuDaWang(inner) => return Run::accelerate_step(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn decelerate_step(&self) -> usize {
                match self {
                    Dango::Denia(inner) => return Run::decelerate_step(inner),
                    Dango::Sigrika(inner) => return Run::decelerate_step(inner),
                    Dango::Hiyuki(inner) => return Run::decelerate_step(inner),
                    Dango::Cartethyia(inner) => return Run::decelerate_step(inner),
                    Dango::Phoebe(inner) => return Run::decelerate_step(inner),
                    Dango::LuukHerssen(inner) => return Run::decelerate_step(inner),
                    Dango::BuDaWang(inner) => return Run::decelerate_step(inner),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn step<R>(&mut self, track: &mut Track, map: &Map, rng: &mut R) -> bool
            where
                R: Rng + ?Sized,
            {
                match self {
                    Dango::Denia(inner) => return Run::step::<R>(inner, track, map, rng),
                    Dango::Sigrika(inner) => {
                        return Run::step::<R>(inner, track, map, rng);
                    }
                    Dango::Hiyuki(inner) => return Run::step::<R>(inner, track, map, rng),
                    Dango::Cartethyia(inner) => {
                        return Run::step::<R>(inner, track, map, rng);
                    }
                    Dango::Phoebe(inner) => return Run::step::<R>(inner, track, map, rng),
                    Dango::LuukHerssen(inner) => {
                        return Run::step::<R>(inner, track, map, rng);
                    }
                    Dango::BuDaWang(inner) => {
                        return Run::step::<R>(inner, track, map, rng);
                    }
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn make_step<R>(
                &mut self,
                n: usize,
                track: &mut Track,
                map: &Map,
                rng: &mut R,
            ) -> bool
            where
                R: Rng + ?Sized,
            {
                match self {
                    Dango::Denia(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                    Dango::Sigrika(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                    Dango::Hiyuki(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                    Dango::Cartethyia(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                    Dango::Phoebe(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                    Dango::LuukHerssen(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                    Dango::BuDaWang(inner) => {
                        return Run::make_step::<R>(inner, n, track, map, rng);
                    }
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn before_run(&mut self, _track: &mut Track) {
                match self {
                    Dango::Denia(inner) => return Run::before_run(inner, _track),
                    Dango::Sigrika(inner) => return Run::before_run(inner, _track),
                    Dango::Hiyuki(inner) => return Run::before_run(inner, _track),
                    Dango::Cartethyia(inner) => return Run::before_run(inner, _track),
                    Dango::Phoebe(inner) => return Run::before_run(inner, _track),
                    Dango::LuukHerssen(inner) => return Run::before_run(inner, _track),
                    Dango::BuDaWang(inner) => return Run::before_run(inner, _track),
                }
            }
            #[inline]
            #[allow(unused_braces)]
            fn after_run(&mut self, _track: &mut Track) {
                match self {
                    Dango::Denia(inner) => return Run::after_run(inner, _track),
                    Dango::Sigrika(inner) => return Run::after_run(inner, _track),
                    Dango::Hiyuki(inner) => return Run::after_run(inner, _track),
                    Dango::Cartethyia(inner) => return Run::after_run(inner, _track),
                    Dango::Phoebe(inner) => return Run::after_run(inner, _track),
                    Dango::LuukHerssen(inner) => return Run::after_run(inner, _track),
                    Dango::BuDaWang(inner) => return Run::after_run(inner, _track),
                }
            }
        }
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
}
mod track {
    use std::fmt::Write;
    use unicode_width::UnicodeWidthStr;
    use crate::dangos::{RefDango, Run};
    pub const TRACK_LEN: usize = 32;
    pub enum PointType {
        Accelerate,
        Decelerate,
        Hole,
        Common,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PointType {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PointType::Accelerate => "Accelerate",
                    PointType::Decelerate => "Decelerate",
                    PointType::Hole => "Hole",
                    PointType::Common => "Common",
                },
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for PointType {}
    #[automatically_derived]
    impl ::core::clone::Clone for PointType {
        #[inline]
        fn clone(&self) -> PointType {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PointType {}
    pub type Map = [PointType; TRACK_LEN];
    pub type Point = Vec<RefDango>;
    pub type Track = [Point; TRACK_LEN];
    /// 初始地图
    pub fn init_map() -> Map {
        let mut points = [PointType::Common; TRACK_LEN];
        points[2] = PointType::Accelerate;
        points[5] = PointType::Hole;
        points[9] = PointType::Decelerate;
        points[10] = PointType::Accelerate;
        points[15] = PointType::Accelerate;
        points[19] = PointType::Hole;
        points[22] = PointType::Accelerate;
        points[27] = PointType::Decelerate;
        points
    }
    /// 初始团子赛道
    pub fn init_track(dangos: &[RefDango]) -> Track {
        let mut track = [const { ::alloc::vec::Vec::new() }; TRACK_LEN];
        track[0] = dangos.iter().rev().cloned().collect();
        track
    }
    pub fn show_track(round: usize, track: &Track) {
        const ROW_NUM: usize = 8;
        const COL_NUM: usize = 4;
        const COL_WIDTH: usize = 42;
        const LINE_WIDTH: usize = COL_WIDTH * COL_NUM;
        const SEP_NUM: usize = (LINE_WIDTH - 4) / 2;
        let mut track_state = String::new();
        for row in 0..ROW_NUM {
            for col in 0..COL_NUM {
                let idx = row + col * 8;
                let point = &track[idx];
                let mut cell = ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("{0:2}: ", idx + 1))
                });
                for dango in point {
                    (&mut cell)
                        .write_fmt(
                            format_args!(
                                "{0}({1}) ",
                                dango.borrow().shortname(),
                                dango.borrow().get_arrive_count(),
                            ),
                        )
                        .unwrap();
                }
                let cell_width = UnicodeWidthStr::width(cell.as_str());
                (&mut track_state).write_fmt(format_args!("{0}", cell)).unwrap();
                if cell_width < COL_WIDTH {
                    (&mut track_state)
                        .write_fmt(
                            format_args!("{0}", " ".repeat(COL_WIDTH - cell_width)),
                        )
                        .unwrap();
                }
            }
            track_state.push('\n');
        }
        {
            ::std::io::_print(
                format_args!(
                    "{0} {1:02} {2}\n{3}\n",
                    "=".repeat(SEP_NUM),
                    round,
                    "=".repeat(SEP_NUM),
                    track_state,
                ),
            );
        };
    }
}
mod utils {
    pub fn split_first<T>(mut array: Vec<T>) -> (T, Vec<T>) {
        if !!array.is_empty() {
            ::core::panicking::panic("assertion failed: !array.is_empty()")
        }
        let res = array.split_off(1);
        let first = array.pop().unwrap();
        (first, res)
    }
}
#[doc(inline)]
///A macro to be used by [`ambassador::Delegate`] to delegate [`Rng`]
use _ambassador_impl_Rng as ambassador_impl_Rng;
#[doc(hidden)]
#[allow(non_snake_case)]
mod ambassador_impl_Rng {}
#[delegate(Rng)]
enum MyRng {
    StdRng(StdRng),
    ThreadRng(ThreadRng),
}
#[allow(non_snake_case)]
mod ambassador_module_Rng_for_MyRng {
    use super::*;
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub trait MatchRng<ambassador_X: Rng>: Rng {}
    #[allow(non_camel_case_types)]
    impl<ambassador_X: Rng, ambassador_Y: Rng> MatchRng<ambassador_X> for ambassador_Y {}
    impl Rng for MyRng
    where
        ThreadRng: Rng,
        StdRng: MatchRng<ThreadRng>,
    {
        #[inline]
        #[allow(unused_braces)]
        fn next_u32(&mut self) -> u32 {
            match self {
                MyRng::StdRng(inner) => return Rng::next_u32(inner),
                MyRng::ThreadRng(inner) => return Rng::next_u32(inner),
            }
        }
        #[inline]
        #[allow(unused_braces)]
        fn next_u64(&mut self) -> u64 {
            match self {
                MyRng::StdRng(inner) => return Rng::next_u64(inner),
                MyRng::ThreadRng(inner) => return Rng::next_u64(inner),
            }
        }
        #[inline]
        #[allow(unused_braces)]
        fn fill_bytes(&mut self, dst: &mut [u8]) {
            match self {
                MyRng::StdRng(inner) => return Rng::fill_bytes(inner, dst),
                MyRng::ThreadRng(inner) => return Rng::fill_bytes(inner, dst),
            }
        }
    }
}
impl From<StdRng> for MyRng {
    fn from(rng: StdRng) -> Self {
        MyRng::StdRng(rng)
    }
}
impl From<ThreadRng> for MyRng {
    fn from(rng: ThreadRng) -> Self {
        MyRng::ThreadRng(rng)
    }
}
pub struct GameState {
    rng: MyRng,
    map: Map,
    dangos: Vec<RefDango>,
    track: Track,
    before_run_dangos: Vec<RefDango>,
    after_run_dangos: Vec<RefDango>,
    budawang: RefDango,
    round: usize,
}
impl GameState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rng: MyRng,
        map: Map,
        dangos: Vec<RefDango>,
        track: Track,
        before_run_dangos: Vec<RefDango>,
        after_run_dangos: Vec<RefDango>,
        budawang: RefDango,
        round: usize,
    ) -> Self {
        Self {
            rng,
            map,
            dangos,
            track,
            before_run_dangos,
            after_run_dangos,
            budawang,
            round,
        }
    }
}
fn init_game() -> GameState {
    let mut rng = StdRng::seed_from_u64(0);
    let map = init_map();
    let budawang = dangos::new_bu_da_wang();
    let sigrika = dangos::new_sigrika();
    let mut dangos = ::alloc::boxed::box_assume_init_into_vec_unsafe(
        ::alloc::intrinsics::write_box_via_move(
            ::alloc::boxed::Box::new_uninit(),
            [
                dangos::new_denia(),
                sigrika.clone(),
                dangos::new_hiyuki(),
                dangos::new_cartethyia(),
                dangos::new_phoebe(),
                dangos::new_luuk_herssen(),
            ],
        ),
    );
    let before_run_dangos = ::alloc::boxed::box_assume_init_into_vec_unsafe(
        ::alloc::intrinsics::write_box_via_move(
            ::alloc::boxed::Box::new_uninit(),
            [sigrika],
        ),
    );
    let after_run_dangos = ::alloc::vec::Vec::new();
    dangos.shuffle(&mut rng);
    dangos
        .iter()
        .rev()
        .enumerate()
        .for_each(|(idx, dango)| dango.borrow_mut().set_pos((0, idx)));
    let track = init_track(&dangos);
    GameState::new(
        rng.into(),
        map,
        dangos,
        track,
        before_run_dangos,
        after_run_dangos,
        budawang,
        0,
    )
}
/// @return: (结束时比赛状态, 布大王团子)
fn one_game(first_half_finish_state: Option<GameState>) -> GameState {
    let from_beginning = first_half_finish_state.is_none();
    let GameState {
        mut rng,
        map,
        mut dangos,
        mut track,
        before_run_dangos,
        mut after_run_dangos,
        budawang,
        round: _,
    } = first_half_finish_state.unwrap_or_else(init_game);
    if !from_beginning {
        budawang.borrow_mut().set_pos((TRACK_LEN - 1, 0));
        dangos
            .iter()
            .for_each(|dango| {
                let mut dango = dango.borrow_mut();
                dango.reset();
                dango.increase_target_arrive_count();
            });
    }
    let mut round = 0;
    let mut arrived = false;
    'GameLoop: while !arrived {
        round += 1;
        if round == 3 {
            dangos.push(budawang.clone());
            after_run_dangos.push(budawang.clone());
            track[TRACK_LEN - 1].push(budawang.clone());
        }
        if round != 1 || !from_beginning {
            dangos.shuffle(&mut rng);
            for dango in before_run_dangos.iter() {
                dango.borrow_mut().before_run(&mut track);
            }
        }
        for dango in dangos.iter() {
            arrived = dango.borrow_mut().step(&mut track, &map, &mut rng);
            if arrived {
                break 'GameLoop;
            }
        }
        for dango in after_run_dangos.iter() {
            dango.borrow_mut().after_run(&mut track);
        }
    }
    {
        dangos.retain(|dango| !is_budawang(dango));
        after_run_dangos.retain(|dango| !is_budawang(dango));
        let budawang = budawang.borrow();
        let (x, _) = budawang.get_pos();
        track[x].remove(0);
        track[x]
            .iter()
            .enumerate()
            .for_each(|(idx, dango)| dango.borrow_mut().set_pos((x, idx)));
    }
    GameState::new(
        rng,
        map,
        dangos,
        track,
        before_run_dangos,
        after_run_dangos,
        budawang,
        round,
    )
}
#[allow(unused)]
fn sort_by_dangos(dangos: &mut [RefDango]) {
    dangos
        .sort_by(|a, b| {
            let (x_a, y_a) = a.borrow().get_pos();
            let (x_b, y_b) = b.borrow().get_pos();
            if x_a == x_b { y_a.cmp(&y_b) } else { x_a.cmp(&x_b) }
        });
    dangos.reverse();
}
#[allow(unused)]
fn sort_by_track(track: &Track) -> Vec<RefDango> {
    track.iter().rev().flat_map(|point| point.iter().rev()).cloned().collect()
}
#[allow(unused)]
fn show_rank(dangos: &[RefDango]) {
    let mut rank_info = String::with_capacity(10 * dangos.len());
    for dango in dangos.iter() {
        let dango = dango.borrow();
        let (x, y) = dango.get_pos();
        (&mut rank_info)
            .write_fmt(format_args!("{0}({1}, {2}), ", dango.shortname(), x, y))
            .unwrap();
    }
    {
        ::std::io::_print(format_args!("{0}\n", rank_info));
    };
}
fn main() {
    let half_state = one_game(None);
    let mut _finish_state = one_game(Some(half_state));
}

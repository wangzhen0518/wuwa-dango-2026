#![feature(prelude_import)]
#![allow(unused)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::fmt::Write;
use ambassador::{Delegate, delegatable_trait_remote};
use rand::{
    SeedableRng, TryRng, rngs::{StdRng, ThreadRng},
    seq::SliceRandom,
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
        fn step<R>(
            &self,
            _dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
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
                tail[1..].iter_mut().for_each(|dango| dango.increase_arrive_count());
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
                        let (left, _) = track.split_at_mut(target_x + 1);
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
                    }
                }
            }
            self.set_extra(0);
            track[target_x]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, dango)| dango.set_pos((target_x, idx)));
            self.get_arrive_count() == self.get_target_arrive_count() - 1
                && target_x == track.len() - 1
        }
        fn before_run(&self, _dangos: &[Dango], _track: &mut Track) {}
        fn after_run(&self, _dangos: &[Dango], _track: &mut Track) {}
    }
    #[doc(inline)]
    ///A macro to be used by [`ambassador::Delegate`] to delegate [`Run`]
    pub use _ambassador_impl_Run as ambassador_impl_Run;
    #[doc(hidden)]
    #[allow(non_snake_case)]
    pub mod ambassador_impl_Run {}
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
    #[automatically_derived]
    impl ::core::fmt::Debug for Denia {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "n",
                "last_dice",
                "pos",
                "extra",
                "arrive_count",
                "target_arrive_count",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.n,
                &self.last_dice,
                &self.pos,
                &self.extra,
                &self.arrive_count,
                &&self.target_arrive_count,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "Denia", names, values)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Denia {
        #[inline]
        fn clone(&self) -> Denia {
            Denia {
                n: ::core::clone::Clone::clone(&self.n),
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
                n: 0,
                last_dice: 0,
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for RefCell<Denia> {
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
            self.borrow_mut().extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }
        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos;
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
        fn reset(&self) {
            let mut self_mut_inner = self.borrow_mut();
            self_mut_inner.last_dice = 0;
            self_mut_inner.extra = 0;
            self_mut_inner.n = 0;
        }
        fn step<R>(
            &self,
            _dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
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
    pub struct Sigrika {
        n: usize,
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
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Sigrika",
                "n",
                &self.n,
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
                n: ::core::clone::Clone::clone(&self.n),
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
                n: 0,
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for RefCell<Sigrika> {
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
            self.borrow_mut().extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }
        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos;
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
        fn before_run(&self, dangos: &[Dango], track: &mut Track) {
            let self_inner = self.borrow();
            let mut ahead_dangos: Vec<_> = dangos
                .iter()
                .filter(|dango| {
                    !#[allow(non_exhaustive_omitted_patterns)]
                    match dango {
                        Dango::BuDaWang(_) | Dango::Sigrika(_) => true,
                        _ => false,
                    }
                        && dango
                            .get_arrive_count()
                            .cmp(&self_inner.arrive_count)
                            .then(dango.get_pos().cmp(&self_inner.pos))
                            .is_gt()
                })
                .cloned()
                .collect();
            if !ahead_dangos.is_empty() {
                sort_dangos(&mut ahead_dangos);
                ahead_dangos
                    .iter_mut()
                    .rev()
                    .take(2)
                    .for_each(|dango| {
                        let target_extra = dango.get_extra() - 1;
                        dango.set_extra(target_extra);
                    });
            }
        }
    }
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
    #[automatically_derived]
    impl ::core::fmt::Debug for Hiyuki {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "n",
                "meeted",
                "pos",
                "extra",
                "arrive_count",
                "target_arrive_count",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.n,
                &self.meeted,
                &self.pos,
                &self.extra,
                &self.arrive_count,
                &&self.target_arrive_count,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Hiyuki",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Hiyuki {
        #[inline]
        fn clone(&self) -> Hiyuki {
            Hiyuki {
                n: ::core::clone::Clone::clone(&self.n),
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
            self.borrow_mut().extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }
        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos;
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
        fn reset(&self) {
            let mut self_inner = self.borrow_mut();
            self_inner.meeted = false;
            self_inner.extra = 0;
            self_inner.n = 0;
        }
        fn step<R>(
            &self,
            dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
        where
            R: Rng + ?Sized,
        {
            let (old_x, _) = self.get_pos();
            let mut self_mut_inner = self.borrow_mut();
            self_mut_inner.meeted = !self_mut_inner.meeted
                && is_budawang(&track[old_x][0]);
            self_mut_inner.extra = self_mut_inner.meeted as isize;
            drop(self_mut_inner);
            let arrived = self.make_step(track, map, rng);
            let mut self_mut_inner = self.borrow_mut();
            if !arrived && !self_mut_inner.meeted {
                let (new_x, _) = self_mut_inner.pos;
                if new_x > old_x {
                    self_mut_inner.meeted = has_budawang(&track[old_x + 1..=new_x]);
                } else {
                    self_mut_inner.meeted = has_budawang(&track[old_x + 1..])
                        || has_budawang(&track[..=new_x]);
                }
            }
            arrived
        }
    }
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
    #[automatically_derived]
    impl ::core::fmt::Debug for Cartethyia {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "n",
                "has_been_last",
                "pos",
                "extra",
                "arrive_count",
                "target_arrive_count",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.n,
                &self.has_been_last,
                &self.pos,
                &self.extra,
                &self.arrive_count,
                &&self.target_arrive_count,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Cartethyia",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Cartethyia {
        #[inline]
        fn clone(&self) -> Cartethyia {
            Cartethyia {
                n: ::core::clone::Clone::clone(&self.n),
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
                n: 0,
                has_been_last: false,
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
        fn is_last(&self, dangos: &[Dango]) -> bool {
            let after_self_dangos: Vec<_> = dangos
                .iter()
                .filter(|dango| {
                    !#[allow(non_exhaustive_omitted_patterns)]
                    match dango {
                        Dango::BuDaWang(_) | Dango::Cartethyia(_) => true,
                        _ => false,
                    }
                        && dango
                            .get_arrive_count()
                            .cmp(&self.arrive_count)
                            .then(dango.get_pos().cmp(&self.pos))
                            .is_lt()
                })
                .cloned()
                .collect();
            if true {
                #[allow(clippy::needless_bool)]
                if after_self_dangos.is_empty() { true } else { false }
            } else {
                after_self_dangos.is_empty()
            }
        }
    }
    impl Run for RefCell<Cartethyia> {
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
            self.borrow_mut().extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }
        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos;
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
        fn reset(&self) {
            let mut self_mut_inner = self.borrow_mut();
            self_mut_inner.has_been_last = false;
            self_mut_inner.extra = 0;
            self_mut_inner.n = 0;
        }
        fn step<R>(
            &self,
            dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
        where
            R: Rng + ?Sized,
        {
            let mut self_mut_inner = self.borrow_mut();
            if self_mut_inner.has_been_last
                && rng.random_bool(Cartethyia::EXTRA_ADVANCE_PROB)
            {
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
    pub struct Phoebe {
        n: usize,
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
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Phoebe",
                "n",
                &self.n,
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
                n: ::core::clone::Clone::clone(&self.n),
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
                n: 0,
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for RefCell<Phoebe> {
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
            self.borrow_mut().extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }
        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos;
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
        fn step<R>(
            &self,
            _dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
        where
            R: Rng + ?Sized,
        {
            if rng.random_bool(Phoebe::EXTRA_ADVANCE_PROB) {
                self.borrow_mut().extra += 1;
            }
            self.make_step(track, map, rng)
        }
    }
    pub struct LuukHerssen {
        n: usize,
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
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "LuukHerssen",
                "n",
                &self.n,
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
                n: ::core::clone::Clone::clone(&self.n),
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
                n: 0,
                pos: (0, 0),
                extra: 0,
                arrive_count: 0,
                target_arrive_count: 1,
            }
        }
    }
    impl Run for RefCell<LuukHerssen> {
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
            self.borrow_mut().extra = extra;
        }
        fn get_pos(&self) -> (usize, usize) {
            self.borrow().pos
        }
        fn set_pos(&self, pos: (usize, usize)) {
            self.borrow_mut().pos = pos;
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
        fn accelerate_step(&self) -> usize {
            4
        }
        fn decelerate_step(&self) -> usize {
            2
        }
    }
    pub struct BuDaWang {
        n: usize,
        pos: (usize, usize),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for BuDaWang {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "BuDaWang",
                "n",
                &self.n,
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
                n: ::core::clone::Clone::clone(&self.n),
                pos: ::core::clone::Clone::clone(&self.pos),
            }
        }
    }
    impl BuDaWang {
        const BUDAWANG_DICE: [usize; 6] = [1, 2, 3, 4, 5, 6];
        pub fn new() -> Self {
            Self {
                n: 0,
                pos: (TRACK_LEN - 1, 0),
            }
        }
        fn leave_last_dango(&self, dangos: &[Dango]) -> bool {
            let (x, _) = self.pos;
            let mut other_dangos: Vec<_> = dangos
                .iter()
                .filter(|dango| {
                    !#[allow(non_exhaustive_omitted_patterns)]
                    match dango {
                        Dango::BuDaWang(_) => true,
                        _ => false,
                    }
                })
                .cloned()
                .collect();
            sort_dangos(&mut other_dangos);
            let last_dango = other_dangos.last().expect("Always can get dango");
            let (last_x, _last_y) = last_dango.get_pos();
            last_x > x
        }
    }
    impl Run for RefCell<BuDaWang> {
        fn roll<R>(&self, rng: &mut R)
        where
            R: Rng + ?Sized,
        {
            let n = *BuDaWang::BUDAWANG_DICE.choose(rng).expect("Roll failed");
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
            self.borrow_mut().pos = pos;
        }
        fn get_arrive_count(&self) -> usize {
            0
        }
        fn increase_arrive_count(&self) {}
        fn get_target_arrive_count(&self) -> usize {
            0
        }
        fn increase_target_arrive_count(&self) {}
        fn step<R>(
            &self,
            _dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
        where
            R: Rng + ?Sized,
        {
            let n = self.get_n();
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
                .iter_mut()
                .enumerate()
                .skip(1)
                .for_each(|(idx, dango)| dango.set_pos((target_x, idx)));
            false
        }
        fn after_run(&self, dangos: &[Dango], track: &mut Track) {
            if self.borrow().leave_last_dango(dangos) {
                let (x, _) = self.get_pos();
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
    pub fn is_budawang(dango: &Dango) -> bool {
        #[allow(non_exhaustive_omitted_patterns)]
        match dango {
            Dango::BuDaWang(_) => true,
            _ => false,
        }
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
    pub fn sort_dangos(dangos: &mut [Dango]) {
        dangos
            .sort_by(|a, b| {
                a.get_arrive_count()
                    .cmp(&b.get_arrive_count())
                    .then(a.get_pos().cmp(&b.get_pos()))
            });
        dangos.reverse();
    }
    pub enum Dango {
        Denia(Rc<RefCell<Denia>>),
        Sigrika(Rc<RefCell<Sigrika>>),
        Hiyuki(Rc<RefCell<Hiyuki>>),
        Cartethyia(Rc<RefCell<Cartethyia>>),
        Phoebe(Rc<RefCell<Phoebe>>),
        LuukHerssen(Rc<RefCell<LuukHerssen>>),
        BuDaWang(Rc<RefCell<BuDaWang>>),
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
    impl Run for Dango {
        fn reset(&self) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.reset(),
                Dango::Sigrika(ref_cell) => ref_cell.reset(),
                Dango::Hiyuki(ref_cell) => ref_cell.reset(),
                Dango::Cartethyia(ref_cell) => ref_cell.reset(),
                Dango::Phoebe(ref_cell) => ref_cell.reset(),
                Dango::LuukHerssen(ref_cell) => ref_cell.reset(),
                Dango::BuDaWang(ref_cell) => ref_cell.reset(),
            }
        }
        fn roll<R>(&self, rng: &mut R)
        where
            R: Rng + ?Sized,
        {
            match self {
                Dango::Denia(ref_cell) => ref_cell.roll(rng),
                Dango::Sigrika(ref_cell) => ref_cell.roll(rng),
                Dango::Hiyuki(ref_cell) => ref_cell.roll(rng),
                Dango::Cartethyia(ref_cell) => ref_cell.roll(rng),
                Dango::Phoebe(ref_cell) => ref_cell.roll(rng),
                Dango::LuukHerssen(ref_cell) => ref_cell.roll(rng),
                Dango::BuDaWang(ref_cell) => ref_cell.roll(rng),
            }
        }
        fn get_n(&self) -> usize {
            match self {
                Dango::Denia(ref_cell) => ref_cell.get_n(),
                Dango::Sigrika(ref_cell) => ref_cell.get_n(),
                Dango::Hiyuki(ref_cell) => ref_cell.get_n(),
                Dango::Cartethyia(ref_cell) => ref_cell.get_n(),
                Dango::Phoebe(ref_cell) => ref_cell.get_n(),
                Dango::LuukHerssen(ref_cell) => ref_cell.get_n(),
                Dango::BuDaWang(ref_cell) => ref_cell.get_n(),
            }
        }
        fn set_n(&self, n: usize) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.set_n(n),
                Dango::Sigrika(ref_cell) => ref_cell.set_n(n),
                Dango::Hiyuki(ref_cell) => ref_cell.set_n(n),
                Dango::Cartethyia(ref_cell) => ref_cell.set_n(n),
                Dango::Phoebe(ref_cell) => ref_cell.set_n(n),
                Dango::LuukHerssen(ref_cell) => ref_cell.set_n(n),
                Dango::BuDaWang(ref_cell) => ref_cell.set_n(n),
            }
        }
        fn get_extra(&self) -> isize {
            match self {
                Dango::Denia(ref_cell) => ref_cell.get_extra(),
                Dango::Sigrika(ref_cell) => ref_cell.get_extra(),
                Dango::Hiyuki(ref_cell) => ref_cell.get_extra(),
                Dango::Cartethyia(ref_cell) => ref_cell.get_extra(),
                Dango::Phoebe(ref_cell) => ref_cell.get_extra(),
                Dango::LuukHerssen(ref_cell) => ref_cell.get_extra(),
                Dango::BuDaWang(ref_cell) => ref_cell.get_extra(),
            }
        }
        fn set_extra(&self, extra: isize) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.set_extra(extra),
                Dango::Sigrika(ref_cell) => ref_cell.set_extra(extra),
                Dango::Hiyuki(ref_cell) => ref_cell.set_extra(extra),
                Dango::Cartethyia(ref_cell) => ref_cell.set_extra(extra),
                Dango::Phoebe(ref_cell) => ref_cell.set_extra(extra),
                Dango::LuukHerssen(ref_cell) => ref_cell.set_extra(extra),
                Dango::BuDaWang(ref_cell) => ref_cell.set_extra(extra),
            }
        }
        fn get_pos(&self) -> (usize, usize) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.get_pos(),
                Dango::Sigrika(ref_cell) => ref_cell.get_pos(),
                Dango::Hiyuki(ref_cell) => ref_cell.get_pos(),
                Dango::Cartethyia(ref_cell) => ref_cell.get_pos(),
                Dango::Phoebe(ref_cell) => ref_cell.get_pos(),
                Dango::LuukHerssen(ref_cell) => ref_cell.get_pos(),
                Dango::BuDaWang(ref_cell) => ref_cell.get_pos(),
            }
        }
        fn set_pos(&self, pos: (usize, usize)) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.set_pos(pos),
                Dango::Sigrika(ref_cell) => ref_cell.set_pos(pos),
                Dango::Hiyuki(ref_cell) => ref_cell.set_pos(pos),
                Dango::Cartethyia(ref_cell) => ref_cell.set_pos(pos),
                Dango::Phoebe(ref_cell) => ref_cell.set_pos(pos),
                Dango::LuukHerssen(ref_cell) => ref_cell.set_pos(pos),
                Dango::BuDaWang(ref_cell) => ref_cell.set_pos(pos),
            }
        }
        fn get_arrive_count(&self) -> usize {
            match self {
                Dango::Denia(ref_cell) => ref_cell.get_arrive_count(),
                Dango::Sigrika(ref_cell) => ref_cell.get_arrive_count(),
                Dango::Hiyuki(ref_cell) => ref_cell.get_arrive_count(),
                Dango::Cartethyia(ref_cell) => ref_cell.get_arrive_count(),
                Dango::Phoebe(ref_cell) => ref_cell.get_arrive_count(),
                Dango::LuukHerssen(ref_cell) => ref_cell.get_arrive_count(),
                Dango::BuDaWang(ref_cell) => ref_cell.get_arrive_count(),
            }
        }
        fn increase_arrive_count(&self) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.increase_arrive_count(),
                Dango::Sigrika(ref_cell) => ref_cell.increase_arrive_count(),
                Dango::Hiyuki(ref_cell) => ref_cell.increase_arrive_count(),
                Dango::Cartethyia(ref_cell) => ref_cell.increase_arrive_count(),
                Dango::Phoebe(ref_cell) => ref_cell.increase_arrive_count(),
                Dango::LuukHerssen(ref_cell) => ref_cell.increase_arrive_count(),
                Dango::BuDaWang(ref_cell) => ref_cell.increase_arrive_count(),
            }
        }
        fn get_target_arrive_count(&self) -> usize {
            match self {
                Dango::Denia(ref_cell) => ref_cell.get_target_arrive_count(),
                Dango::Sigrika(ref_cell) => ref_cell.get_target_arrive_count(),
                Dango::Hiyuki(ref_cell) => ref_cell.get_target_arrive_count(),
                Dango::Cartethyia(ref_cell) => ref_cell.get_target_arrive_count(),
                Dango::Phoebe(ref_cell) => ref_cell.get_target_arrive_count(),
                Dango::LuukHerssen(ref_cell) => ref_cell.get_target_arrive_count(),
                Dango::BuDaWang(ref_cell) => ref_cell.get_target_arrive_count(),
            }
        }
        fn increase_target_arrive_count(&self) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.increase_target_arrive_count(),
                Dango::Sigrika(ref_cell) => ref_cell.increase_target_arrive_count(),
                Dango::Hiyuki(ref_cell) => ref_cell.increase_target_arrive_count(),
                Dango::Cartethyia(ref_cell) => ref_cell.increase_target_arrive_count(),
                Dango::Phoebe(ref_cell) => ref_cell.increase_target_arrive_count(),
                Dango::LuukHerssen(ref_cell) => ref_cell.increase_target_arrive_count(),
                Dango::BuDaWang(ref_cell) => ref_cell.increase_target_arrive_count(),
            }
        }
        fn accelerate_step(&self) -> usize {
            match self {
                Dango::Denia(ref_cell) => ref_cell.accelerate_step(),
                Dango::Sigrika(ref_cell) => ref_cell.accelerate_step(),
                Dango::Hiyuki(ref_cell) => ref_cell.accelerate_step(),
                Dango::Cartethyia(ref_cell) => ref_cell.accelerate_step(),
                Dango::Phoebe(ref_cell) => ref_cell.accelerate_step(),
                Dango::LuukHerssen(ref_cell) => ref_cell.accelerate_step(),
                Dango::BuDaWang(ref_cell) => ref_cell.accelerate_step(),
            }
        }
        fn decelerate_step(&self) -> usize {
            match self {
                Dango::Denia(ref_cell) => ref_cell.decelerate_step(),
                Dango::Sigrika(ref_cell) => ref_cell.decelerate_step(),
                Dango::Hiyuki(ref_cell) => ref_cell.decelerate_step(),
                Dango::Cartethyia(ref_cell) => ref_cell.decelerate_step(),
                Dango::Phoebe(ref_cell) => ref_cell.decelerate_step(),
                Dango::LuukHerssen(ref_cell) => ref_cell.decelerate_step(),
                Dango::BuDaWang(ref_cell) => ref_cell.decelerate_step(),
            }
        }
        fn step<R>(
            &self,
            dangos: &[Dango],
            track: &mut Track,
            map: &Map,
            rng: &mut R,
        ) -> bool
        where
            R: Rng + ?Sized,
        {
            match self {
                Dango::Denia(ref_cell) => ref_cell.step(dangos, track, map, rng),
                Dango::Sigrika(ref_cell) => ref_cell.step(dangos, track, map, rng),
                Dango::Hiyuki(ref_cell) => ref_cell.step(dangos, track, map, rng),
                Dango::Cartethyia(ref_cell) => ref_cell.step(dangos, track, map, rng),
                Dango::Phoebe(ref_cell) => ref_cell.step(dangos, track, map, rng),
                Dango::LuukHerssen(ref_cell) => ref_cell.step(dangos, track, map, rng),
                Dango::BuDaWang(ref_cell) => ref_cell.step(dangos, track, map, rng),
            }
        }
        fn make_step<R>(&self, track: &mut Track, map: &Map, rng: &mut R) -> bool
        where
            R: Rng + ?Sized,
        {
            match self {
                Dango::Denia(ref_cell) => ref_cell.make_step(track, map, rng),
                Dango::Sigrika(ref_cell) => ref_cell.make_step(track, map, rng),
                Dango::Hiyuki(ref_cell) => ref_cell.make_step(track, map, rng),
                Dango::Cartethyia(ref_cell) => ref_cell.make_step(track, map, rng),
                Dango::Phoebe(ref_cell) => ref_cell.make_step(track, map, rng),
                Dango::LuukHerssen(ref_cell) => ref_cell.make_step(track, map, rng),
                Dango::BuDaWang(ref_cell) => ref_cell.make_step(track, map, rng),
            }
        }
        fn before_run(&self, dangos: &[Dango], track: &mut Track) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.before_run(dangos, track),
                Dango::Sigrika(ref_cell) => ref_cell.before_run(dangos, track),
                Dango::Hiyuki(ref_cell) => ref_cell.before_run(dangos, track),
                Dango::Cartethyia(ref_cell) => ref_cell.before_run(dangos, track),
                Dango::Phoebe(ref_cell) => ref_cell.before_run(dangos, track),
                Dango::LuukHerssen(ref_cell) => ref_cell.before_run(dangos, track),
                Dango::BuDaWang(ref_cell) => ref_cell.before_run(dangos, track),
            }
        }
        fn after_run(&self, dangos: &[Dango], track: &mut Track) {
            match self {
                Dango::Denia(ref_cell) => ref_cell.after_run(dangos, track),
                Dango::Sigrika(ref_cell) => ref_cell.after_run(dangos, track),
                Dango::Hiyuki(ref_cell) => ref_cell.after_run(dangos, track),
                Dango::Cartethyia(ref_cell) => ref_cell.after_run(dangos, track),
                Dango::Phoebe(ref_cell) => ref_cell.after_run(dangos, track),
                Dango::LuukHerssen(ref_cell) => ref_cell.after_run(dangos, track),
                Dango::BuDaWang(ref_cell) => ref_cell.after_run(dangos, track),
            }
        }
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
}
mod track {
    use std::fmt::Write;
    use unicode_width::UnicodeWidthStr;
    use crate::dangos::{Dango, Run, sort_dangos};
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
    impl PointType {
        pub fn shortname(&self) -> &'static str {
            match self {
                PointType::Accelerate => "A",
                PointType::Decelerate => "D",
                PointType::Hole => "H",
                PointType::Common => "C",
            }
        }
    }
    pub type Map = [PointType; TRACK_LEN];
    pub type Point = Vec<Dango>;
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
    pub fn init_track(dangos: &[Dango]) -> Track {
        let mut track = [const { ::alloc::vec::Vec::new() }; TRACK_LEN];
        track[0] = dangos.iter().rev().cloned().collect();
        track
    }
    pub fn show_track(round: usize, dangos: &[Dango], track: &Track, map: &Map) {
        const ROW_NUM: usize = 8;
        const COL_NUM: usize = 4;
        const COL_WIDTH: usize = 45;
        const LINE_WIDTH: usize = COL_WIDTH * COL_NUM;
        const SEP_NUM: usize = (LINE_WIDTH - 4) / 2;
        let mut track_state = ::alloc::__export::must_use({
            ::alloc::fmt::format(
                format_args!(
                    "{0} {1:02} {2}\n",
                    "=".repeat(SEP_NUM),
                    round,
                    "=".repeat(SEP_NUM),
                ),
            )
        });
        static DANGO_SEP: &str = " -> ";
        for dango in dangos.iter() {
            (&mut track_state)
                .write_fmt(
                    format_args!(
                        "{0}({1}){2}",
                        dango.shortname(),
                        dango.get_n(),
                        DANGO_SEP,
                    ),
                )
                .expect("Write failed");
        }
        track_state.truncate(track_state.len() - DANGO_SEP.len());
        track_state.push('\n');
        for row in 0..ROW_NUM {
            for col in 0..COL_NUM {
                let idx = row + col * 8;
                let point = &track[idx];
                let mut cell = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("{0:2}({1}): ", idx, map[idx].shortname()),
                    )
                });
                for dango in point {
                    (&mut cell)
                        .write_fmt(
                            format_args!(
                                "{0}({1}) ",
                                dango.shortname(),
                                dango.get_arrive_count(),
                            ),
                        )
                        .expect("Write failed");
                }
                let cell_width = UnicodeWidthStr::width(cell.as_str());
                (&mut track_state)
                    .write_fmt(format_args!("{0}", cell))
                    .expect("Write failed");
                if cell_width < COL_WIDTH {
                    (&mut track_state)
                        .write_fmt(
                            format_args!("{0}", " ".repeat(COL_WIDTH - cell_width)),
                        )
                        .expect("Write failed");
                }
            }
            track_state.push('\n');
        }
        {
            ::std::io::_print(format_args!("{0}\n", track_state));
        };
    }
    pub fn sort_by_track(track: &Track) -> Vec<Dango> {
        let mut dangos: Vec<_> = track
            .iter()
            .rev()
            .flat_map(|point| point.iter().rev())
            .cloned()
            .collect();
        sort_dangos(&mut dangos);
        dangos
    }
}
mod utils {
    pub fn split_first<T>(mut array: Vec<T>) -> (T, Vec<T>) {
        if !!array.is_empty() {
            ::core::panicking::panic("assertion failed: !array.is_empty()")
        }
        let res = array.split_off(1);
        let first = array
            .pop()
            .expect("As array is not empty, always can get the first element");
        (first, res)
    }
}
use crate::{
    dangos::{Dango, Run, is_budawang},
    track::{Map, TRACK_LEN, Track, init_map, init_track, show_track},
};
#[doc(inline)]
///A macro to be used by [`ambassador::Delegate`] to delegate [`TryRng`]
use _ambassador_impl_TryRng as ambassador_impl_TryRng;
#[doc(hidden)]
#[allow(non_snake_case)]
mod ambassador_impl_TryRng {}
#[delegate(TryRng)]
pub enum MyRng {
    StdRng(Box<StdRng>),
    ThreadRng(ThreadRng),
}
#[allow(non_snake_case)]
mod ambassador_module_TryRng_for_MyRng {
    use super::*;
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub trait MatchTryRng<
        ambassador_X: TryRng,
    >: TryRng<Error = <ambassador_X as TryRng>::Error> {}
    #[allow(non_camel_case_types)]
    impl<
        ambassador_X: TryRng,
        ambassador_Y: TryRng<Error = <ambassador_X as TryRng>::Error>,
    > MatchTryRng<ambassador_X> for ambassador_Y {}
    impl TryRng for MyRng
    where
        ThreadRng: TryRng,
        Box<StdRng>: MatchTryRng<ThreadRng>,
    {
        type Error = <ThreadRng as TryRng>::Error;
        #[inline]
        #[allow(unused_braces)]
        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            match self {
                MyRng::StdRng(inner) => return TryRng::try_next_u32(inner),
                MyRng::ThreadRng(inner) => return TryRng::try_next_u32(inner),
            }
        }
        #[inline]
        #[allow(unused_braces)]
        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            match self {
                MyRng::StdRng(inner) => return TryRng::try_next_u64(inner),
                MyRng::ThreadRng(inner) => return TryRng::try_next_u64(inner),
            }
        }
        #[inline]
        #[allow(unused_braces)]
        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            match self {
                MyRng::StdRng(inner) => return TryRng::try_fill_bytes(inner, dst),
                MyRng::ThreadRng(inner) => return TryRng::try_fill_bytes(inner, dst),
            }
        }
    }
}
impl From<StdRng> for MyRng {
    fn from(rng: StdRng) -> Self {
        MyRng::StdRng(Box::new(rng))
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
    dangos: Vec<Dango>,
    track: Track,
    before_run_dangos: Vec<Dango>,
    after_run_dangos: Vec<Dango>,
    budawang: Dango,
    round: usize,
}
impl GameState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rng: MyRng,
        map: Map,
        dangos: Vec<Dango>,
        track: Track,
        before_run_dangos: Vec<Dango>,
        after_run_dangos: Vec<Dango>,
        budawang: Dango,
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
    let args: Vec<_> = std::env::args().collect();
    let mut rng = if args.len() > 1 {
        let seed: u64 = args[1].parse().unwrap();
        {
            ::std::io::_print(format_args!("seed = {0}\n", seed));
        };
        StdRng::seed_from_u64(seed).into()
    } else {
        rand::rng().into()
    };
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
    dangos.iter_mut().rev().enumerate().for_each(|(idx, dango)| dango.set_pos((0, idx)));
    let track = init_track(&dangos);
    GameState::new(
        rng,
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
        dangos
            .iter_mut()
            .for_each(|dango| {
                dango.reset();
                dango.increase_target_arrive_count();
            });
    }
    show_track(0, &dangos, &track, &map);
    let mut round = 0;
    let mut arrived = false;
    'GameLoop: while !arrived {
        round += 1;
        if round == 3 {
            budawang.set_pos((TRACK_LEN - 1, 0));
            dangos.push(budawang.clone());
            after_run_dangos.push(budawang.clone());
            track[TRACK_LEN - 1].insert(0, budawang.clone());
            track[TRACK_LEN - 1]
                .iter_mut()
                .enumerate()
                .skip(1)
                .for_each(|(idx, dango)| dango.set_pos((TRACK_LEN - 1, idx)));
        }
        if round != 1 || !from_beginning {
            dangos.shuffle(&mut rng);
            for dango in before_run_dangos.iter() {
                dango.before_run(&dangos, &mut track);
            }
        }
        for dango in dangos.iter_mut() {
            dango.roll(&mut rng);
        }
        for dango in dangos.iter() {
            arrived = dango.step(&dangos, &mut track, &map, &mut rng);
            if arrived {
                break 'GameLoop;
            }
        }
        for dango in after_run_dangos.iter_mut() {
            dango.after_run(&dangos, &mut track);
        }
        show_track(round, &dangos, &track, &map);
    }
    {
        dangos.retain(|dango| !is_budawang(dango));
        after_run_dangos.retain(|dango| !is_budawang(dango));
        let (x, _) = budawang.get_pos();
        track[x].remove(0);
        track[x].iter_mut().enumerate().for_each(|(idx, dango)| dango.set_pos((x, idx)));
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
fn show_rank(dangos: &[Dango]) {
    let mut rank_info = String::with_capacity(10 * dangos.len());
    for dango in dangos.iter() {
        let (x, y) = dango.get_pos();
        (&mut rank_info)
            .write_fmt(format_args!("{0}({1}, {2}), ", dango.shortname(), x, y))
            .expect("Write failed");
    }
    {
        ::std::io::_print(format_args!("{0}\n", rank_info));
    };
}
fn main() {
    if true {
        {
            ::std::io::_print(format_args!("Start first half game\n"));
        };
    }
    let half_state = one_game(None);
    show_track(half_state.round, &half_state.dangos, &half_state.track, &half_state.map);
    if true {
        {
            ::std::io::_print(format_args!("Start second half game\n"));
        };
    }
    let mut finish_state = one_game(Some(half_state));
    show_track(
        finish_state.round,
        &finish_state.dangos,
        &finish_state.track,
        &finish_state.map,
    );
}

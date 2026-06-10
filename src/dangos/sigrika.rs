use std::{cell::RefCell, rc::Rc};

use crate::{
    dangos::{Dango, Run, impl_run_helper, sort_dangos},
    track::Track,
};

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

impl Default for Sigrika {
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

impl Run for RefCell<Sigrika> {
    impl_run_helper!();

    fn before_run(&self, dangos: &[Dango], _track: &mut Track) {
        let self_inner = self.borrow();
        // 收集除布大王外，领先于自己的团子
        let mut ahead_dangos: Vec<_> = dangos
            .iter()
            .filter(|dango| {
                !matches!(dango, Dango::BuDaWang(_) | Dango::Sigrika(_))
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
            ahead_dangos.iter_mut().rev().take(2).for_each(|dango| {
                let target_extra = dango.get_extra() - 1;
                dango.set_extra(target_extra);
            });
        }
    }
}

pub fn new_sigrika(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
) -> Rc<RefCell<Sigrika>> {
    Rc::new(RefCell::new(Sigrika {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
    }))
}

pub fn default_sigrika() -> Rc<RefCell<Sigrika>> {
    Rc::new(RefCell::new(Sigrika::default()))
}

#[cfg(test)]
mod tests {
    use crate::dangos::tests::*;

    use super::*;

    #[test]
    fn test_sigrika_debuffs_top_two_ahead() {
        const DENIA_POS: usize = 10;
        const HIYUKI_POS: usize = 7;
        const SIGRIKA_POS: usize = 3;
        const PHOEBE_POS: usize = 1;

        let mut track = dummy_track_no_dangos();

        let sigrika = default_sigrika();
        let denia = Dango::default_denia();
        let hiyuki = Dango::default_hiyuki();
        let phoebe = Dango::default_phoebe();
        let dangos = [
            sigrika.clone().into(),
            denia.clone(),
            hiyuki.clone(),
            phoebe.clone(),
        ];

        denia.set_pos((DENIA_POS, 0));
        hiyuki.set_pos((HIYUKI_POS, 0));
        sigrika.set_pos((SIGRIKA_POS, 0));
        phoebe.set_pos((PHOEBE_POS, 0));

        track[DENIA_POS].push(denia.clone());
        track[HIYUKI_POS].push(hiyuki.clone());
        track[SIGRIKA_POS].push(sigrika.clone().into());
        track[PHOEBE_POS].push(phoebe.clone());

        sigrika.before_run(&dangos, &mut track);

        assert_eq!(denia.get_extra(), -1, "top 1 ahead should be debuffed");
        assert_eq!(hiyuki.get_extra(), -1, "top 2 ahead should be debuffed");
        assert_eq!(phoebe.get_extra(), 0, "behind dango should not be debuffed");
    }
}

use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::dangos::{Run, impl_run_helper};

static MORNYE_DICE: [usize; 6] = [3, 2, 1, 3, 2, 1];

#[derive(Debug, Clone)]
pub struct Mornye {
    n: usize,
    /// (track position, height)
    pos: (usize, usize),
    /// buff 或 debuff 效果
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    next_dice_index: usize,
}

impl Default for Mornye {
    fn default() -> Self {
        Self {
            n: 0,
            pos: (0, 0),
            extra: 0,
            arrive_count: 0,
            target_arrive_count: 1,
            next_dice_index: 0,
        }
    }
}

impl Run for RefCell<Mornye> {
    impl_run_helper!();

    fn roll<R>(&self, _rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let mut self_mut_inner = self.borrow_mut();
        let n = MORNYE_DICE[self_mut_inner.next_dice_index];
        self_mut_inner.n = n;
        self_mut_inner.next_dice_index = (self_mut_inner.next_dice_index + 1) % MORNYE_DICE.len();
    }

    fn reset(&self) {
        let mut self_mut_inner = self.borrow_mut();
        self_mut_inner.extra = 0;
        self_mut_inner.n = 0;
        self_mut_inner.next_dice_index = 0;
    }
}

pub fn new_mornye(
    n: usize,
    pos: (usize, usize),
    extra: isize,
    arrive_count: usize,
    target_arrive_count: usize,
    next_dice_index: usize,
) -> Rc<RefCell<Mornye>> {
    Rc::new(RefCell::new(Mornye {
        n,
        pos,
        extra,
        arrive_count,
        target_arrive_count,
        next_dice_index,
    }))
}

pub fn default_mornye() -> Rc<RefCell<Mornye>> {
    Rc::new(RefCell::new(Mornye::default()))
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    #[test]
    fn test_denia_consecutive_dice_bonus() {
        let mut rng = StdRng::seed_from_u64(0);
        let mornye = default_mornye();
        let n_seq: Vec<usize> = (0..9)
            .map(|_| {
                mornye.roll(&mut rng);
                mornye.get_n()
            })
            .collect();
        let tgt_n_seq: Vec<usize> = (0..9)
            .map(|idx| MORNYE_DICE[idx % MORNYE_DICE.len()])
            .collect();
        assert_eq!(n_seq, tgt_n_seq);
    }
}

use crate::{
    dangos::{budawang::default_budawang, denia::default_denia},
    track::TRACK_LEN,
};
use rand::{SeedableRng, rngs::StdRng};

use super::*;

pub(in crate::dangos) fn dummy_map() -> Map {
    [PointType::Common; TRACK_LEN]
}

pub(in crate::dangos) fn dummy_track_no_dangos() -> Track {
    [const { vec![] }; TRACK_LEN]
}

// ────────── 基础测试 ──────────

#[test]
fn test_get_n() {
    let d = default_denia();
    assert_eq!(d.get_n(), 0);
}

#[test]
fn test_set_n() {
    let d = default_denia();
    d.set_n(3);
    assert_eq!(d.get_n(), 3);
}

#[test]
fn test_roll_in_range() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = default_denia();
    d.roll(&mut rng);
    let n = d.get_n();
    assert!((1..=3).contains(&n), "common dice must be 1..=3, got {n}");
}

#[test]
fn test_budawang_roll_in_range() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = default_budawang();
    d.roll(&mut rng);
    let n = d.get_n();
    assert!((1..=6).contains(&n), "budawang dice must be 1..=6, got {n}");
}

#[test]
fn test_reset() {
    let d = default_denia();
    d.set_n(3);
    d.set_extra(5);
    d.reset();
    assert_eq!(d.get_n(), 0);
    assert_eq!(d.get_extra(), 0);
}

#[test]
fn test_get_set_extra() {
    let d = default_denia();
    d.set_extra(3);
    assert_eq!(d.get_extra(), 3);
    d.set_extra(-2);
    assert_eq!(d.get_extra(), -2);
}

#[test]
fn test_budawang_extra_is_always_zero() {
    let d = default_budawang();
    assert_eq!(d.get_extra(), 0);
    d.set_extra(99);
    assert_eq!(d.get_extra(), 0);
}

#[test]
fn test_shortname_fullname() {
    assert_eq!(DangoKind::Denia.shortname(), "达");
    assert_eq!(DangoKind::Sigrika.fullname(), "西格莉卡");
    assert_eq!(DangoKind::BuDaWang.fullname(), "布大王");
}

// ────────── pos 与 arrive_count ──────────

#[test]
fn test_set_pos() {
    let d = default_denia();
    d.set_pos((5, 2));
    assert_eq!(d.get_pos(), (5, 2));
}

#[test]
fn test_arrive_count() {
    let d = default_denia();
    assert_eq!(d.get_arrive_count(), 0);
    d.increase_arrive_count();
    assert_eq!(d.get_arrive_count(), 1);
    assert_eq!(d.get_target_arrive_count(), 1);
    d.increase_target_arrive_count();
    assert_eq!(d.get_target_arrive_count(), 2);
}

// ────────── make_step 基本移动 ──────────

#[test]
fn test_make_step_basic_movement() {
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(42);

    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    // 将 d 放在 track[0] 的 pos，roll 值为 2
    let denia = Dango::default_denia();
    denia.set_pos((0, 0));
    track[0].push(denia.clone());

    denia.set_n(2);

    let arrived = denia.make_step(&mut track, &map, &mut rng);
    // 移动到 2 处
    let (x, _) = denia.get_pos();
    assert_eq!(x, 2, "should move to track[2]");
    assert!(!arrived, "should not arrive at finish");
    assert_eq!(denia.get_extra(), 0, "extra should reset after move");
}

#[test]
fn test_make_step_wrap_around() {
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(42);
    let denia = Dango::default_denia();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    denia.set_pos((TRACK_LEN - 2, 0));
    denia.set_n(3);
    denia.set_extra(0);
    // remain_arrive_count > 1 时才能跨越终点
    denia.increase_target_arrive_count();
    track[TRACK_LEN - 2].push(denia.clone());

    let _ = denia.make_step(&mut track, &map, &mut rng);
    // target_x = 33 >= 32 → increase_arrive, 33%32 = 1
    assert!(
        denia.get_arrive_count() >= 1,
        "should cross finish line when moving past track end"
    );
    let (x, _) = denia.get_pos();
    assert_eq!(x, 1, "after wrapping, should land at track[1]");
}

// ────────── 赛道效果 ──────────

#[test]
fn test_accelerate_step() {
    let denia = Dango::default_denia();
    let map = [PointType::Accelerate; TRACK_LEN];
    let mut track = dummy_track_no_dangos();
    let mut rng = StdRng::seed_from_u64(42);

    denia.set_pos((0, 0));
    denia.set_n(1);
    track[0].push(denia.clone());

    let _ = denia.make_step(&mut track, &map, &mut rng);
    let (x, _) = denia.get_pos();
    assert_eq!(
        x, 2,
        "accelerate should push forward 1 extra step (base 1 + accelerate=1)"
    );
}

#[test]
fn test_decelerate_step() {
    let denia = Dango::default_denia();
    let map = [PointType::Decelerate; TRACK_LEN];
    let mut track = dummy_track_no_dangos();
    let mut rng = StdRng::seed_from_u64(42);

    denia.set_pos((2, 0));
    denia.set_n(1);
    track[2].push(denia.clone());

    let _ = denia.make_step(&mut track, &map, &mut rng);
    let (x, _) = denia.get_pos();
    // pos=(2,0), split_off(0) → tail=[d], track[2]=[]
    // target_x=2+1=3, append → track[3]=[d], target_y=0
    // Decelerate: new_x=3-1=2, split_at_mut(3) left=[0..2] right=[3..31]
    // left[2].append(right[0]) → track[2]=[d]
    assert_eq!(
        x, 2,
        "decelerate should push backward 1 step (target_x=3 - decelerate=1)"
    );
}

#[test]
fn test_hole_shuffles() {
    let mut rng = StdRng::seed_from_u64(42);

    let map = [PointType::Hole; TRACK_LEN];
    let mut track = dummy_track_no_dangos();

    let denia = Dango::default_denia();
    let sigrika = Dango::default_sigrika();

    denia.set_pos((0, 0));
    sigrika.set_pos((0, 1));

    track[0].push(denia.clone());
    track[0].push(sigrika.clone());

    denia.set_n(1);
    denia.make_step(&mut track, &map, &mut rng);

    let (x, _) = denia.get_pos();
    assert_eq!(x, 1, "step 1");

    let (x, _) = sigrika.get_pos();
    assert_eq!(x, 1, "moved by denia");

    // hole should shuffle but both dangos should still be in track[target_x]
    assert_eq!(track[x].len(), 2, "both dangos should remain after hole");
}

// ────────── BuDaWang 测试 ──────────

#[test]
fn test_budawang_reverse_movement() {
    let budawang = Dango::default_budawang();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    budawang.set_pos((10, 0));
    budawang.set_n(3);
    track[10].push(budawang.clone());

    let mut rng = StdRng::seed_from_u64(42);
    let _ = budawang.step(&[], &mut track, &map, &mut rng);
    let (x, _) = budawang.get_pos();
    // BuDaWang moves backward: 10 - 3 = 7
    assert_eq!(x, 7, "budawang should move backward to track[7]");
}

#[test]
fn test_budawang_carry_dangos() {
    let budawang = Dango::default_budawang();
    let denia = Dango::default_denia();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    // BuDaWang at (20, 0), Denia on same cell at (20, 1)
    budawang.set_pos((20, 0));
    denia.set_pos((20, 1));
    budawang.set_n(5);
    track[20].push(budawang.clone());
    track[20].push(denia.clone());

    let mut rng = StdRng::seed_from_u64(42);
    let _ = budawang.step(&[], &mut track, &map, &mut rng);

    // BuDaWang moved to 15, Denia should be carried there
    let (bdx, _bdy) = budawang.get_pos();
    assert_eq!(bdx, 15, "budawang moved to 15");
    assert!(
        track[bdx].len() >= 2,
        "denia should be carried to same track cell"
    );
}

// ────────── sort_dangos ──────────
#[test]
fn test_sort_dangos_dummy_case() {
    let denia = Dango::default_denia(); // arrive=0, pos=(5,0)
    let sigrika = Dango::default_sigrika(); // arrive=0, pos=(3,0)
    let hiyuki = Dango::default_hiyuki(); // arrive=1, pos=(1,0)

    denia.set_pos((5, 0));
    sigrika.set_pos((3, 0));
    hiyuki.set_pos((1, 0));
    hiyuki.increase_arrive_count();

    let mut list = vec![denia.clone(), sigrika.clone(), hiyuki.clone()];
    sort_dangos(&mut list);

    // sorted by arrive_count desc, then pos desc
    let actual: Vec<_> = list.iter().map(std::mem::discriminant).collect();
    let expected: Vec<_> = [hiyuki, denia, sigrika]
        .iter()
        .map(std::mem::discriminant)
        .collect();
    assert_eq!(actual, expected); // Hiyuki (arrive=1) > Denia (pos=5) > Sigrika (pos=3)
}

#[test]
fn test_sort_dangos_real_case() {
    let mut src = [
        Dango::new_cartethyia(1, (30, 1), 0, 0, 1, false),
        Dango::new_phoebe(1, (28, 0), 0, 0, 1),
        Dango::new_luukherssen(1, (30, 0), 0, 0, 1),
        Dango::new_sigrika(1, (25, 0), 0, 0, 1),
        Dango::new_denia(1, (31, 0), 0, 0, 1, 1),
        Dango::new_hiyuki(1, (24, 0), 0, 0, 1, false),
    ];

    sort_dangos(&mut src);

    let target = [
        Dango::new_denia(1, (31, 0), 0, 0, 1, 1),
        Dango::new_cartethyia(1, (30, 1), 0, 0, 1, false),
        Dango::new_luukherssen(1, (30, 0), 0, 0, 1),
        Dango::new_phoebe(1, (28, 0), 0, 0, 1),
        Dango::new_sigrika(1, (25, 0), 0, 0, 1),
        Dango::new_hiyuki(1, (24, 0), 0, 0, 1, false),
    ];

    assert_eq!(src, target);
}

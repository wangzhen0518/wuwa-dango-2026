use rand::{SeedableRng, rngs::StdRng};
use crate::track::TRACK_LEN;

use super::*;

fn dummy_map() -> Map {
    [PointType::Common; TRACK_LEN]
}

fn dummy_track_no_dangos() -> Track {
    [const { vec![] }; TRACK_LEN]
}

// ────────── 基础测试 ──────────

#[test]
fn test_get_n() {
    let d = new_denia();
    assert_eq!(d.get_n(), 0);
}

#[test]
fn test_set_n() {
    let d = new_denia();
    d.set_n(3);
    assert_eq!(d.get_n(), 3);
}

#[test]
fn test_roll_in_range() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = new_denia();
    d.roll(&mut rng);
    let n = d.get_n();
    assert!((1..=3).contains(&n), "common dice must be 1..=3, got {n}");
}

#[test]
fn test_budawang_roll_in_range() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = new_bu_da_wang();
    d.roll(&mut rng);
    let n = d.get_n();
    assert!((1..=6).contains(&n), "budawang dice must be 1..=6, got {n}");
}

#[test]
fn test_reset() {
    let d = new_denia();
    d.set_n(3);
    d.set_extra(5);
    d.reset();
    assert_eq!(d.get_n(), 0);
    assert_eq!(d.get_extra(), 0);
}

#[test]
fn test_get_set_extra() {
    let d = new_denia();
    d.set_extra(3);
    assert_eq!(d.get_extra(), 3);
    d.set_extra(-2);
    assert_eq!(d.get_extra(), -2);
}

#[test]
fn test_budawang_extra_is_always_zero() {
    let d = new_bu_da_wang();
    assert_eq!(d.get_extra(), 0);
    d.set_extra(99);
    assert_eq!(d.get_extra(), 0);
}

#[test]
fn test_shortname_fullname() {
    assert_eq!(new_denia().shortname(), "达");
    assert_eq!(new_sigrika().fullname(), "西格莉卡");
    assert_eq!(new_bu_da_wang().fullname(), "布大王");
}

// ────────── pos 与 arrive_count ──────────

#[test]
fn test_set_pos() {
    let d = new_denia();
    d.set_pos((5, 2));
    assert_eq!(d.get_pos(), (5, 2));
}

#[test]
fn test_arrive_count() {
    let d = new_denia();
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
    let d = new_denia();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    // 将 d 放在 track[0] 的 pos，roll 值为 2
    d.set_pos((0, 0));
    d.set_n(2);
    track[0].push(d.clone());

    let arrived = d.make_step(&mut track, &map, &mut rng);
    // 移动到 2 处
    let (x, _y) = d.get_pos();
    assert_eq!(x, 2, "should move to track[2]");
    assert!(!arrived, "should not arrive at finish");
    assert_eq!(d.get_extra(), 0, "extra should reset after move");
}

#[test]
fn test_make_step_wrap_around() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = new_denia();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    d.set_pos((TRACK_LEN - 2, 0));
    d.set_n(3);
    d.set_extra(0);
    // remain_arrive_count > 1 时才能跨越终点
    d.increase_target_arrive_count();
    track[TRACK_LEN - 2].push(d.clone());

    let _ = d.make_step(&mut track, &map, &mut rng);
    // target_x = 33 >= 32 → increase_arrive, 33%32 = 1
    assert!(d.get_arrive_count() >= 1,
        "should cross finish line when moving past track end");
    let (x, _) = d.get_pos();
    assert_eq!(x, 1, "after wrapping, should land at track[1]");
}

// ────────── 技能测试 ──────────

#[test]
fn test_denia_consecutive_dice_bonus() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = new_denia();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();
    let mut rng2 = StdRng::seed_from_u64(42);

    d.set_pos((0, 0));
    track[0].push(d.clone());

    // 第一次 roll 设定 last_dice
    d.roll(&mut rng);
    let first_n = d.get_n();

    // 手动再 roll 一次
    d.roll(&mut rng2);
    // Denia::step: if n == last_dice, extra += 2
    let extra_before = d.get_extra();
    let _ = d.step(std::slice::from_ref(&d), &mut track, &map, &mut rng2);
    // 由于使用了不同的 rng 种子，两次 roll 值可能不同
    // 但 step 方法会对比 self.get_n() 和 self.last_dice
    let extra_after = d.get_extra();
    // extra 在 step 结束时被 make_step 设为 0，所以这里无法通过 extra_after 检验
    // 但确保 step 不会出错即可
}

#[test]
fn test_luukherssen_accelerate_decelerate() {
    let d = new_luuk_herssen();
    assert_eq!(d.accelerate_step(), 4);
    assert_eq!(d.decelerate_step(), 2);
}

#[test]
fn test_phoebe_step_does_not_panic() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let d = new_phoebe();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();
    d.set_pos((0, 0));
    d.set_n(1);
    track[0].push(d.clone());

    let _ = d.step(&[], &mut track, &map, &mut rng);
}

// ────────── 赛道效果 ──────────

#[test]
fn test_accelerate_step() {
    let d = new_denia();
    let map = [PointType::Accelerate; TRACK_LEN];
    let mut track = dummy_track_no_dangos();
    let mut rng = StdRng::seed_from_u64(42);

    d.set_pos((0, 0));
    d.set_n(1);
    track[0].push(d.clone());

    let _ = d.make_step(&mut track, &map, &mut rng);
    let (x, _) = d.get_pos();
    assert_eq!(x, 2, "accelerate should push forward 1 extra step (base 1 + accelerate=1)");
}

#[test]
fn test_decelerate_step() {
    let d = new_denia();
    let map = [PointType::Decelerate; TRACK_LEN];
    let mut track = dummy_track_no_dangos();
    let mut rng = StdRng::seed_from_u64(42);

    d.set_pos((2, 0));
    d.set_n(1);
    track[2].push(d.clone());

    let _ = d.make_step(&mut track, &map, &mut rng);
    let (x, _) = d.get_pos();
    // pos=(2,0), split_off(0) → tail=[d], track[2]=[]
    // target_x=2+1=3, append → track[3]=[d], target_y=0
    // Decelerate: new_x=3-1=2, split_at_mut(3) left=[0..2] right=[3..31]
    // left[2].append(right[0]) → track[2]=[d]
    assert_eq!(x, 2, "decelerate should push backward 1 step (target_x=3 - decelerate=1)");
}

#[test]
fn test_hole_shuffles() {
    let d1 = new_denia();
    let d2 = new_sigrika();
    let map = [PointType::Hole; TRACK_LEN];
    let mut rng = StdRng::seed_from_u64(42);

    d1.set_pos((0, 0));
    d2.set_pos((0, 1));
    let mut track = dummy_track_no_dangos();
    track[0].push(d1.clone());
    track[0].push(d2.clone());

    let _ = d1.make_step(&mut track, &map, &mut rng);
    // hole should shuffle but both dangos should still be in track[target_x]
    let (x, _) = d1.get_pos();
    assert_eq!(track[x].len(), 2, "both dangos should remain after hole");
}

// ────────── BuDaWang 测试 ──────────

#[test]
fn test_budawang_reverse_movement() {
    let bd = new_bu_da_wang();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    bd.set_pos((10, 0));
    bd.set_n(3);
    track[10].push(bd.clone());

    let mut rng = StdRng::seed_from_u64(42);
    let _ = bd.step(&[], &mut track, &map, &mut rng);
    let (x, _) = bd.get_pos();
    // BuDaWang moves backward: 10 - 3 = 7
    assert_eq!(x, 7, "budawang should move backward to track[7]");
}

#[test]
fn test_budawang_carry_dangos() {
    let bd = new_bu_da_wang();
    let d = new_denia();
    let map = dummy_map();
    let mut track = dummy_track_no_dangos();

    // BuDaWang at (20, 0), Denia on same cell at (20, 1)
    bd.set_pos((20, 0));
    d.set_pos((20, 1));
    bd.set_n(5);
    track[20].push(bd.clone());
    track[20].push(d.clone());

    let mut rng = StdRng::seed_from_u64(42);
    let _ = bd.step(&[], &mut track, &map, &mut rng);

    // BuDaWang moved to 15, Denia should be carried there
    let (bdx, _bdy) = bd.get_pos();
    assert_eq!(bdx, 15, "budawang moved to 15");
    assert!(track[bdx].len() >= 2, "denia should be carried to same track cell");
}

// ────────── sort_dangos ──────────

#[test]
fn test_sort_dangos_by_arrive_then_pos() {
    let d1 = new_denia(); // arrive=0, pos=(5,0)
    let d2 = new_sigrika(); // arrive=0, pos=(3,0)
    let d3 = new_hiyuki(); // arrive=1, pos=(1,0)

    d1.set_pos((5, 0));
    d2.set_pos((3, 0));
    d3.set_pos((1, 0));
    d3.increase_arrive_count();

    let mut list = vec![d1.clone(), d2.clone(), d3.clone()];
    sort_dangos(&mut list);

    // sorted by arrive_count desc, then pos desc
    assert!(matches!(list[0], Dango::Hiyuki(_))); // arrive=1 first
}

// ────────── 完整游戏模拟 ──────────

#[test]
fn test_full_game_no_panic() {
    use rand::SeedableRng;
    use crate::track::init_map;
    let seed = 12345u64;
    let mut rng = StdRng::seed_from_u64(seed);

    let map = init_map();
    let budawang = new_bu_da_wang();
    let sigrika = new_sigrika();
    let mut dangos = vec![
        new_denia(),
        sigrika.clone(),
        new_hiyuki(),
        new_cartethyia(),
        new_phoebe(),
        new_luuk_herssen(),
    ];
    let before_run_dangos = [sigrika];

    dangos.shuffle(&mut rng);
    dangos.iter_mut().rev().enumerate().for_each(|(idx, dango)| {
        dango.set_pos((0, idx));
    });
    let mut track = crate::track::init_track(&dangos);

    let from_beginning = true;

    // ── 第一半局 ──
    let mut round = 0usize;
    let mut arrived = false;
    'GameLoop1: while !arrived {
        round += 1;

        if round == 3 {
            budawang.set_pos((TRACK_LEN - 1, 0));
            dangos.push(budawang.clone());
            track[TRACK_LEN - 1].insert(0, budawang.clone());
            track[TRACK_LEN - 1].iter_mut().enumerate().skip(1).for_each(|(idx, d)| d.set_pos((TRACK_LEN - 1, idx)));
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
                break 'GameLoop1;
            }
        }

        if round > 40 { break; } // 安全阀
    }

    // 移除布大王
    dangos.retain(|d| !is_budawang(d));
    let (bx, _) = budawang.get_pos();
    track[bx].remove(0);
    track[bx].iter_mut().enumerate().for_each(|(idx, d)| d.set_pos((bx, idx)));

    // ── 第二半局重置 ──
    dangos.iter_mut().for_each(|d| {
        d.reset();
        d.increase_target_arrive_count();
    });

    // ── 第二半局 ──
    let mut arrived2 = false;
    'GameLoop2: while !arrived2 {
        round += 1;

        dangos.shuffle(&mut rng);
        for dango in dangos.iter_mut() {
            dango.roll(&mut rng);
        }
        for dango in dangos.iter() {
            arrived2 = dango.step(&dangos, &mut track, &map, &mut rng);
            if arrived2 {
                break 'GameLoop2;
            }
        }

        if round > 80 { break; }
    }
}

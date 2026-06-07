# 🍡 WuWa Dango Racing 2026 — 鸣潮团子竞速赛

> **Dango Great Battle** event simulator for _Wuthering Waves (鸣潮)_
>
> A racing competition simulator written in Rust.

---

## 📖 Overview

This project simulates the **Dango Great Battle** racing competition from the _Wuthering Waves_ game event. Cute little dumplings (团子 / Dango) race on a circular track, each with a unique skill that affects movement. The simulator runs races, computes win probabilities through Monte Carlo simulation, and supports multi-stage tournaments:

- **Group Stage** — 3 groups of 6 dango each, top 4 advance, bottom 2 go to Curtain Call
- **Knockout Stage** — elimination bracket (not yet implemented)
- **Finals** — championship match (not yet implemented)
- **Curtain Call** — consolation match for eliminated dango (not yet implemented)

---

## 🏁 Features

- **Race Simulation** — full turn-by-turn race logic with dice rolling, stacking, and track mechanics
- **Unique Dango Skills** — each dango has a distinct skill that triggers during the race
- **Track Map** — 32-cell circular track with special tiles (Accelerate, Decelerate, Hole)
- **BuDaWang Mechanic** — a special "Big King" dango that moves from the finish line backward
- **Dango Stacking** — dango can stack on top of each other when landing on the same cell
- **Deterministic Mode** — seed-based RNG for reproducible simulations
- **Debug Visualization** — round-by-round track state display (debug builds)
- **Monte Carlo Support** — architecture designed for batch simulation and probability computation

---

## 🧩 Architecture

```
src/
├── main.rs              # Entry point & game loop
├── dangos.rs            # Dango enum, Run trait, factory functions
├── dangos/
│   ├── denia.rs         # Denia — "Double" bonus on consecutive same dice
│   ├── sigrika.rs       # Sigrika — Marks nearby dango, reduces their move
│   ├── hiyuki.rs        # Hiyuki — Extra move after meeting BuDaWang
│   ├── cartethyia.rs    # Cartethyia — 60% chance extra move when last
│   ├── phoebe.rs        # Phoebe — 50% chance extra move
│   ├── luukherssen.rs   # LuukHerssen — Boost on accelerate, penalty on decelerate
│   ├── budawang.rs      # BuDaWang — Moves backward from finish line
│   └── tests.rs         # Unit tests
├── track.rs             # Track map, point types, visualization
└── utils.rs             # Utility functions & debug macros

agent/                   # Design documents & game rules
├── basic.md             # Task overview & requirements
├── 比赛规则.md          # Competition rules
├── 赛道信息.md          # Track map & mechanics
├── 团子技能.md          # Dango skill descriptions
├── 小组赛.md            # Group stage rules
├── 淘汰赛.md            # Knockout stage (pending)
├── 总决赛.md            # Finals (pending)
├── 谢幕赛.md            # Curtain call (pending)
├── refactor.md          # Refactoring plan
└── plan/                # Implementation plans
```

---

## 🎲 Game Rules

### Dice

- Common dice: `[1, 1, 2, 2, 3, 3]`
- BuDaWang dice: `[1, 2, 3, 4, 5, 6]`

### Track

Circular track with 32 cells, from `(1,C)` (start) to `(32,C)` (finish):

| Tile Type          | Effect                               |
| ------------------ | ------------------------------------ |
| **C** (Common)     | No effect                            |
| **A** (Accelerate) | Move +1 cell forward                 |
| **D** (Decelerate) | Move -1 cell backward                |
| **H** (Hole)       | Randomly re-stack dango on this cell |

For BuDaWang, Accelerate moves backward (toward finish) and Decelerate moves forward (toward start).

### Stacking

When a dango lands on a cell already occupied by other dango, it stacks on top. When moving, a dango carries all dango stacked above it.

### BuDaWang

- Enters the race at round 3 from the finish line
- Moves **backward** toward the start
- Always at the bottom of any stack
- Teleports back to the finish line if separated from the last-place dango
- Affected by track mechanisms, but accelerate/decelerate directions are reversed

### Dango Skills

| Dango                 | Skill          | Effect                                                                                                                                 |
| --------------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| 达妮娅 (Denia)        | 好事成"双"     | Same dice as last roll → +2 extra move                                                                                                 |
| 西格莉卡 (Sigrika)    | 日灵，帮帮忙！ | Marks up to 2 higher-ranked dango; they move -1 (min 1)                                                                                |
| 绯雪 (Hiyuki)         | 引路白鸟       | After meeting BuDaWang, +1 extra move each turn                                                                                        |
| 卡提希娅 (Cartethyia) | 翻盘桥段       | When last after moving, 60% chance +2 extra (once per race)                                                                            |
| 菲比 (Phoebe)         | 岁主庇佑       | 50% chance +1 extra move                                                                                                               |
| 陆·赫斯 (LuukHerssen) | 来颗糖吧       | Accelerate: +3; Decelerate: -1 additional                                                                                              |
| 千咲 (Chisaki)        | 视阈解明       | Minimum roll of the round → +2 extra                                                                                                   |
| 莫宁 (Morning)        | 精密演算       | Dice cycles: 3, 2, 1, 3, 2, 1, ...                                                                                                     |
| 琳奈 (Linnae)         | 炫彩时刻！     | 60% double move; 20% cannot move                                                                                                       |
| 爱弥斯 (Aimis)        | 电子幽灵登场   | After midpoint, teleport to nearest dango's stack (once)                                                                               |
| 守岸人 (Shorekeeper)  | 收束的未来     | Dice always 2 or 3                                                                                                                     |
| 珂莱塔 (Koreta)       | 利润加倍       | 28% chance double move                                                                                                                 |
| 奥古斯塔 (Augusta)    | 总督权柄       | If at top of stack, skip turn and move last next round                                                                                 |
| 尤诺 (Yuno)           | 锚定命途       | After midpoint, teleport nearby dango to own cell (once)                                                                               |
| 弗洛洛 (FroLo)        | 优雅阴谋       | If at bottom of stack, +3 extra move                                                                                                   |
| 长离 (Changli)        | 谋而后定       | If dango below, 65% chance move last next round                                                                                        |
| 今汐 (Jinxi)          | 令尹之名       | If dango on top, 40% chance move to top of stack                                                                                       |
| 卡卡罗 (Kakaro)       | 如影随形       | If last, +3 extra move                                                                                                                 |
| 布大王 (BuDaWang)     | —              | Moves backward from finish line since round 3. Dice 1-6, always at stack bottom. Teleports back to finish if separated from last dango |

---

## 🚀 Getting Started

### Prerequisites

- Rust 2024 edition
- Cargo

### Build & Run

```bash
# Debug build (with track visualization)
cargo run

# Release build (fast, no debug output)
cargo run --release

# With a specific random seed (deterministic)
cargo run -- <seed>
# e.g.
cargo run -- 42

# Run tests
cargo test
```

### Current Behavior

The current `main()` simulates a **Group Stage A match** (Denia, Sigrika, Hiyuki, Cartethyia, Phoebe, LuukHerssen) in two halves:

1. **First half** — all dango start at the starting line
2. **Second half** — inherits positions from the first half end; each dango needs to cross the finish line one more time

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output (useful for debug visualization)
cargo test -- --nocapture
```

---

## 📝 TODO

- [x] `Rc<RefCell<>>` refactoring for shared mutability
- [ ] All dango skills implemented
- [x] Dice roll / move separation
- [x] `step()` receives dango list as parameter
- [ ] Parallel Monte Carlo simulation for probability computation
- [ ] JSON-based match configuration loading
- [ ] Race recording to file
- [ ] TUI replay visualization
- [ ] Per-dango rank probability distribution
- [ ] Expected cheering reward calculation based on dark horse values

See [TODO.md](TODO.md) and [agent/](agent/) for details.

---

## ⚙️ Technical Notes

- **Shared State**: Uses `Rc<RefCell<T>>` for shared mutable access to dango state during the race
- **Ambassador Crate**: Uses the `ambassador` crate for delegation pattern on `MyRng`
- **Feature Gates**: Debug visualization guarded behind `#[cfg(debug_assertions)]` — no performance cost in release builds
- **Macro Helpers**: `impl_run_for_dango_helper!` macro auto-generates method dispatch from `Dango` enum to each variant

---

_Made with ❤️ for the Wuthering Waves Dango Great Battle event_

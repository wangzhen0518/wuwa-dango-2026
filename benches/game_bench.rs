use criterion::{Criterion, criterion_group, criterion_main};

use wuwa_dango_2026::game::simulate_game;

fn bench_simulate_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulate_game");
    group.bench_function("n=1_000_000", |b| b.iter(|| simulate_game(1_000_000)));

    group.finish();
}

criterion_group!(benches, bench_simulate_game);
criterion_main!(benches);
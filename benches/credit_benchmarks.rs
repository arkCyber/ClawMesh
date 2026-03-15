use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn credit_calculation_benchmark(c: &mut Criterion) {
    c.bench_function("credit_tier_calculation", |b| {
        b.iter(|| {
            // Benchmark tier calculation
            let score = black_box(500);
            clawmesh_credit::tier::get_reputation_tier(score)
        });
    });
}

fn credit_score_bounds_benchmark(c: &mut Criterion) {
    c.bench_function("credit_score_bounds_check", |b| {
        b.iter(|| {
            // Benchmark bounds checking
            let score = black_box(750);
            clawmesh_credit::calculator::clamp_credit_score(score)
        });
    });
}

criterion_group!(benches, credit_calculation_benchmark, credit_score_bounds_benchmark);
criterion_main!(benches);

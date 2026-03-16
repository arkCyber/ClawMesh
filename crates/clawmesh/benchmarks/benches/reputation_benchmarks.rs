/// Reputation System Performance Benchmarks (DO-178C Level A)
/// 
/// Measures performance of reputation calculation and queries

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clawmesh_reputation::reputation::calculate_reputation_score;
use clawmesh_reputation::models::ReputationLevel;

/// Benchmark reputation score calculation
fn bench_calculate_reputation_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("reputation_calculation");
    
    // Test different vote counts
    for vote_count in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(vote_count),
            vote_count,
            |b, &count| {
                b.iter(|| {
                    calculate_reputation_score(
                        black_box(count / 2),
                        black_box(count / 2)
                    )
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark reputation level calculation
fn bench_reputation_level_from_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("reputation_level");
    
    let scores = vec![0, 299, 300, 599, 600, 999, 1000, 1399, 1400, 1799, 1800, 2000];
    
    for score in scores {
        group.bench_with_input(
            BenchmarkId::from_parameter(score),
            &score,
            |b, &s| {
                b.iter(|| ReputationLevel::from_score(black_box(s)));
            },
        );
    }
    
    group.finish();
}

/// Benchmark reputation score with edge cases
fn bench_reputation_edge_cases(c: &mut Criterion) {
    c.bench_function("reputation_min_score", |b| {
        b.iter(|| calculate_reputation_score(black_box(0), black_box(100)));
    });
    
    c.bench_function("reputation_max_score", |b| {
        b.iter(|| calculate_reputation_score(black_box(200), black_box(0)));
    });
    
    c.bench_function("reputation_balanced", |b| {
        b.iter(|| calculate_reputation_score(black_box(50), black_box(50)));
    });
}

/// Benchmark reputation level boundaries
fn bench_reputation_level_boundaries(c: &mut Criterion) {
    let boundary_scores = vec![299, 300, 599, 600, 999, 1000, 1399, 1400, 1799, 1800];
    
    c.bench_function("reputation_level_boundaries", |b| {
        b.iter(|| {
            for &score in &boundary_scores {
                ReputationLevel::from_score(black_box(score));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_calculate_reputation_score,
    bench_reputation_level_from_score,
    bench_reputation_edge_cases,
    bench_reputation_level_boundaries
);

criterion_main!(benches);

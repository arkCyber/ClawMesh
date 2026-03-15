use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn cache_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");
    
    group.bench_function("cache_insert", |b| {
        b.iter(|| {
            let key = black_box("test_key");
            let value = black_box(42);
            clawmesh_cache::CACHE.set_credit(key, value, Some(Duration::from_secs(60)));
        });
    });
    
    group.bench_function("cache_get", |b| {
        // Pre-populate cache
        clawmesh_cache::CACHE.set_credit("bench_key", 100, Some(Duration::from_secs(60)));
        
        b.iter(|| {
            let key = black_box("bench_key");
            clawmesh_cache::CACHE.get_credit(key)
        });
    });
    
    group.finish();
}

fn cache_concurrent_access_benchmark(c: &mut Criterion) {
    c.bench_function("cache_concurrent_reads", |b| {
        // Pre-populate cache
        for i in 0..100 {
            clawmesh_cache::CACHE.set_credit(
                &format!("key_{}", i),
                i,
                Some(Duration::from_secs(60))
            );
        }
        
        b.iter(|| {
            for i in 0..100 {
                let _ = clawmesh_cache::CACHE.get_credit(&format!("key_{}", i));
            }
        });
    });
}

criterion_group!(benches, cache_operations_benchmark, cache_concurrent_access_benchmark);
criterion_main!(benches);

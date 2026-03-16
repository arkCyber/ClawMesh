/// Skills System Performance Benchmarks (DO-178C Level A)
/// 
/// Measures performance of skill validation and security scanning

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clawmesh_skills::security::validate_skill_code;

/// Benchmark skill code validation
fn bench_skill_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("skill_validation");
    
    let test_codes = vec![
        ("small", "def hello(): return 'Hello'"),
        ("medium", "def process(data):\n    result = []\n    for item in data:\n        result.append(item * 2)\n    return result"),
        ("large", &"def complex_function():\n    pass\n".repeat(100)),
    ];
    
    for (name, code) in test_codes {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &code,
            |b, &code| {
                b.iter(|| {
                    let _ = validate_skill_code(black_box(code));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark security scanning
fn bench_security_scanning(c: &mut Criterion) {
    let safe_code = "def calculate(a, b): return a + b";
    let dangerous_code = "import os; os.system('rm -rf /')";
    
    c.bench_function("security_scan_safe", |b| {
        b.iter(|| {
            let _ = validate_skill_code(black_box(safe_code));
        });
    });
    
    c.bench_function("security_scan_dangerous", |b| {
        b.iter(|| {
            let _ = validate_skill_code(black_box(dangerous_code));
        });
    });
}

/// Benchmark skill type conversions
fn bench_skill_type_operations(c: &mut Criterion) {
    use clawmesh_skills::models::SkillType;
    
    c.bench_function("skill_type_from_i32", |b| {
        b.iter(|| {
            for i in 0..4 {
                SkillType::from_i32(black_box(i));
            }
        });
    });
    
    c.bench_function("skill_type_as_str", |b| {
        b.iter(|| {
            SkillType::Builtin.as_str();
            SkillType::Custom.as_str();
            SkillType::Shared.as_str();
            SkillType::External.as_str();
        });
    });
}

criterion_group!(
    benches,
    bench_skill_validation,
    bench_security_scanning,
    bench_skill_type_operations
);

criterion_main!(benches);

//! Benchmarks for custom field validation and building.
//!
//! These benchmarks measure the performance of custom field operations.
//! Note: Full benchmarks require the crate to expose library functions.
//! These are placeholder benchmarks for future development.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

fn bench_hashmap_operations(c: &mut Criterion) {
    c.bench_function("hashmap_insert_100", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..100 {
                map.insert(black_box(i.to_string()), black_box(i.to_string()));
            }
            map
        })
    });
}

fn bench_string_operations(c: &mut Criterion) {
    c.bench_function("string_format_10", |b| {
        b.iter(|| {
            let mut result = Vec::new();
            for i in 0..10 {
                result.push(format!("field_{}", black_box(i)));
            }
            result
        })
    });
}

fn bench_hashset_operations(c: &mut Criterion) {
    use std::collections::HashSet;
    c.bench_function("hashset_insert_100", |b| {
        b.iter(|| {
            let mut set = HashSet::new();
            for i in 0..100 {
                set.insert(black_box(i.to_string()));
            }
            set
        })
    });
}

criterion_group!(
    benches,
    bench_hashmap_operations,
    bench_string_operations,
    bench_hashset_operations
);
criterion_main!(benches);

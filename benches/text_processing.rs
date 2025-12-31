//! Benchmarks for text processing utilities.
//!
//! These benchmarks measure regex performance for text processing operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;

fn bench_regex_compile(c: &mut Criterion) {
    c.bench_function("regex_compile_profile_pattern", |b| {
        b.iter(|| Regex::new(black_box(r"profiles/(\d+)")))
    });
}

fn bench_regex_replace(c: &mut Criterion) {
    let re = Regex::new(r"profiles/(\d+)").unwrap();
    let text = "Check with profiles/123456 about this task and also profiles/789012";

    c.bench_function("regex_replace_profile_urls", |b| {
        b.iter(|| {
            re.replace_all(black_box(text), |caps: &regex::Captures| {
                if let Some(gid_match) = caps.get(1) {
                    format!("@user_{}", gid_match.as_str())
                } else {
                    caps.get(0)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                }
            })
        })
    });
}

fn bench_string_replace(c: &mut Criterion) {
    let text = "Check with profiles/123456 about this task";

    c.bench_function("string_replace_simple", |b| {
        b.iter(|| black_box(text).replace("profiles/123456", "@user_123456"))
    });
}

criterion_group!(
    benches,
    bench_regex_compile,
    bench_regex_replace,
    bench_string_replace
);
criterion_main!(benches);

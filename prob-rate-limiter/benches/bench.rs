//! $ cargo bench --package prob-rate-limit --bench bench
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prob_rate_limiter::ProbRateLimiter;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("prob_rate_limiter", |b| {
        let clock = Instant::now();
        let mut limiter = ProbRateLimiter::new(500.0).unwrap();
        b.iter(|| {
            limiter.check(black_box(1), clock);
        });
    });
    c.bench_function("governor", |b| {
        let governor_limiter = governor::RateLimiter::direct(governor::Quota::per_second(
            NonZeroU32::new(500u32).unwrap(),
        ));
        b.iter(|| {
            let _ = governor_limiter.check();
        });
    });
    c.bench_function("r8limit", |b| {
        let mut r8limit_limiter = r8limit::RateLimiter::new(500, Duration::from_secs(1));
        b.iter(|| {
            r8limit_limiter.attempt();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

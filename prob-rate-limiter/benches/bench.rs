//! $ cargo bench --package prob-rate-limit --bench bench
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prob_rate_limiter::ProbRateLimiter;
use std::ops::Add;
use std::time::{Duration, Instant};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SimpleRateLimiter::check", |b| {
        b.iter(|| {
            let mut clock = Instant::now();
            let mut limiter = ProbRateLimiter::new(500.0).unwrap();
            for _ in 0..1000 {
                for _ in 0..1000 {
                    limiter.check(black_box(1), clock);
                }
                clock = clock.add(Duration::from_secs(10));
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

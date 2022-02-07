//! $ cargo bench --package fair-ratelimit --bench bench
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fair_ratelimit::FairRateLimiter;
use oorandom::Rand32;
use std::ops::Add;
use std::time::{Duration, Instant};

fn criterion_benchmark(c: &mut Criterion) {
    {
        let mut clock = Instant::now();
        let mut limiter = Box::new(
            <FairRateLimiter<u32, 10>>::new(
                Duration::from_secs(1),
                800,
                200,
                Rand32::new(1),
                clock,
            )
            .unwrap(),
        );
        let mut prng = Rand32::new(2);
        for _ in 0..1000 {
            limiter.check(prng.rand_u32() % 11, 1, clock);
            clock = clock.add(Duration::from_millis(10));
        }
        c.bench_function("internal-10", |b| {
            b.iter(|| {
                limiter.check(prng.rand_u32() % 11, black_box(1), clock);
            })
        });
    }
    {
        let mut clock = Instant::now();
        let mut limiter = Box::new(
            <FairRateLimiter<u32, 1_000_000>>::new(
                Duration::from_secs(1),
                800_000,
                200_000,
                Rand32::new(1),
                clock,
            )
            .unwrap(),
        );
        let mut prng = Rand32::new(2);
        for _ in 0..2_000_000 {
            limiter.check(prng.rand_u32() % 1_100_000, 1, clock);
            clock = clock.add(Duration::from_micros(10));
        }
        c.bench_function("public-1m", |b| {
            b.iter(|| {
                limiter.check(prng.rand_u32() % 1_100_000, black_box(1), clock);
            })
        });
    }
    {
        let mut clock = Instant::now();
        let mut limiter = Box::new(
            <FairRateLimiter<u32, 30_000_000>>::new(
                Duration::from_secs(1),
                800_000_000,
                200_000_000,
                Rand32::new(1),
                clock,
            )
            .unwrap(),
        );
        let mut prng = Rand32::new(2);
        for _ in 0..31_000_000 {
            limiter.check(prng.rand_u32() % 31_000_000, 1, clock);
            clock = clock.add(Duration::from_micros(1));
        }
        c.bench_function("ddos-30m", |b| {
            b.iter(|| {
                limiter.check(prng.rand_u32() % 31_000_000, black_box(1), clock);
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

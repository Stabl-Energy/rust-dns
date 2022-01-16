use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fair_ratelimit::RateLimiter;
use oorandom::Rand32;
use std::ops::Add;
use std::time::{Duration, Instant};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("longtail", |b| {
        b.iter(|| {
            let mut clock = Instant::now();
            let mut limiter = RateLimiter::new_custom(125, Rand32::new(1), clock);
            let mut prng = Rand32::new(2);
            for _ in 0..1000 {
                for _ in 0..100 {
                    limiter.check(prng.rand_u32(), black_box(1), clock);
                }
                clock = clock.add(Duration::from_millis(10));
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

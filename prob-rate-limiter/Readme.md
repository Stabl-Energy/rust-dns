[![crates.io version](https://img.shields.io/crates/v/prob-rate-limiter.svg)](https://crates.io/crates/prob-rate-limiter)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/prob-rate-limiter/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# prob-rate-limiter

`ProbRateLimiter` is a *probabilistic* rate limiter.
When load approaches the configured limit,
the struct chooses randomly whether to accept or reject each request.
It adjusts the probability of rejection so throughput is steady around the limit.

TODO: Add graph from the benchmark.

## Use Cases
- Shed load to prevent overload
- Avoid overloading the services you depend on
- Control costs

## Features
- Tiny, uses 44 bytes
- 100% test coverage
- Optimized: 65ns per check, 15M checks per second on an i5-8259U

## Limitations
- Requires a mutable struct.
- Not fair.  Treats all requests equally, regardless of source.
  A client that overloads the server will consume most of the throughput.

## Alternatives
- [r8limit](https://crates.io/crates/r8limit)
  - Uses a sliding window
  - No `unsafe` or deps
- [governor](https://crates.io/crates/governor)
  - Popular
  - Lots of features
  - Good docs
  - Unnecessary `unsafe`
  - Uses non-standard mutex library [`parking_lot`](https://crates.io/crates/parking_lot)
  - Uses a complicated algorithm
- [leaky-bucket](https://crates.io/crates/leaky-bucket)
  - Async tasks can wait for their turn to use a resource.
  - Unsuitable for load shedding because there is no `try_acquire`.

## Related Crates
- [safe-dns](https://crates.io/crates/safe-dns) uses this

## Example
```rust
let mut limiter = ProbRateLimiter::new(10.0).unwrap();
let mut now = Instant::now();
assert!(limiter.check(5, now));
assert!(limiter.check(5, now));
now += Duration::from_secs(1);
assert!(limiter.check(5, now));
assert!(limiter.check(5, now));
now += Duration::from_secs(1);
assert!(limiter.check(5, now));
assert!(limiter.check(5, now));
now += Duration::from_secs(1);
assert!(limiter.check(5, now));
assert!(limiter.check(5, now));
now += Duration::from_secs(1);
assert!(limiter.check(5, now));
assert!(limiter.check(5, now));
assert!(!limiter.check(5, now));
```

## Cargo Geiger Safety Report
```

Metric output format: x/y
    x = unsafe code used by the build
    y = total unsafe code found in the crate

Symbols: 
    🔒  = No `unsafe` usage found, declares #![forbid(unsafe_code)]
    ❓  = No `unsafe` usage found, missing #![forbid(unsafe_code)]
    ☢️  = `unsafe` usage found

Functions  Expressions  Impls  Traits  Methods  Dependency

0/0        0/0          0/0    0/0     0/0      🔒  prob-rate-limiter 0.1.0
0/0        0/0          0/0    0/0     0/0      🔒  └── oorandom 11.1.3

0/0        0/0          0/0    0/0     0/0    

```
## Changelog
- v0.1.0 - Initial version

# TO DO
- Compare performance with `governor`
- Publish

License: Apache-2.0

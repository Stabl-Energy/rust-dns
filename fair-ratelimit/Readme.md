[![crates.io version](https://img.shields.io/crates/v/fair-ratelimit.svg)](https://crates.io/crates/fair-ratelimit)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/fair-ratelimit/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# fair-ratelimit

Use `RateLimiter` struct to detect overload and
fairly shed load from diverse IP addresses, users, or systems.
Prevent denial-of-service (`DoS`) attacks.

## Use Cases
- DNS server: DNS servers must send UDP replies without a handshake.
  Your DNS server could be used in a
  [DNS amplification attacks](https://www.cisa.gov/uscert/ncas/alerts/TA13-088A).
  Use this crate to prevent that.
- Server without handshake: If your server sends large responses without a handshake,
  it could be used in an amplification attack.  Use this crate to prevent that.
- Load balancer: Use this crate in a load balancer to avoid forwarding DoS attacks to
  backend systems.
- API server: Shed load from misbehaving clients
  and keep the API available for other clients.

## Features
- Global throughput limit
- IPv4 & IPv6
- `forbid(unsafe_code)`, depends only on crates that are `forbid(unsafe_code)`
- ?% test coverage
- Optimized

## Limitations

## Alternatives
- [governor](https://crates.io/crates/governor)
  - Popular
  - Lots of features
  - Good docs
  - Unnecessary `unsafe`
  - Uses non-standard mutex library [`parking_lot`](https://crates.io/crates/parking_lot)
- [r8limit](https://crates.io/crates/r8limit)
  - Supports a single bucket.  Usable for unfair load shedding.
  - No `unsafe` or deps
- [leaky-bucket](https://crates.io/crates/leaky-bucket)
  - Async tasks can wait for their turn to use a resource.
  - Unsuitable for load shedding.

## Related Crates
- [safe-dns](https://crates.io/crates/safe-dns) uses this

## Example
```rust
let mut limiter = RateLimiter::new(5);
let mut now = Instant::now();
let key = IpAddrKey::from(Ipv4Addr::new(10,0,0,1));
assert!(limiter.check(key, 4, now));
assert!(limiter.check(key, 4, now));
assert!(!limiter.check(key, 4, now));
now += Duration::from_secs(1);
assert!(limiter.check(key, 4, now));
assert!(!limiter.check(key, 4, now));
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

0/0        0/0          0/0    0/0     0/0      🔒  fair-ratelimit 0.1.0
0/0        0/0          0/0    0/0     0/0      🔒  └── oorandom 11.1.3

0/0        0/0          0/0    0/0     0/0    

```
## Changelog
- v0.1.0 - Initial version

# TO DO
- `const` constructor
- Adjustable max number of keys
- Adjustable tick duration
- Compare performance with `governor`
- Publish
- Simulate bursty traffic
- Measure memory consumption, add to Limitations section

License: Apache-2.0

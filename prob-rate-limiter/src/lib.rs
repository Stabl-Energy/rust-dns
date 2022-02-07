//! [![crates.io version](https://img.shields.io/crates/v/prob-rate-limiter.svg)](https://crates.io/crates/prob-rate-limiter)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/prob-rate-limiter/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # prob-rate-limiter
//!
//! `ProbRateLimiter` is a *probabilistic* rate limiter.
//! When load approaches the configured limit,
//! the struct chooses randomly whether to accept or reject each request.
//! It adjusts the probability of rejection so throughput is steady around the limit.
//!
//! TODO: Add graph from the benchmark.
//!
//! ## Use Cases
//! - Shed load to prevent overload
//! - Avoid overloading the services you depend on
//! - Control costs
//!
//! ## Features
//! - Tiny, uses 44 bytes
//! - 100% test coverage
//! - Optimized: 65ns per check, 15M checks per second on an i5-8259U
//!
//! ## Limitations
//! - Requires a mutable struct.
//! - Not fair.  Treats all requests equally, regardless of source.
//!   A client that overloads the server will consume most of the throughput.
//!
//! ## Alternatives
//! - [r8limit](https://crates.io/crates/r8limit)
//!   - Uses a sliding window
//!   - No `unsafe` or deps
//! - [governor](https://crates.io/crates/governor)
//!   - Popular
//!   - Lots of features
//!   - Good docs
//!   - Unnecessary `unsafe`
//!   - Uses non-standard mutex library [`parking_lot`](https://crates.io/crates/parking_lot)
//!   - Uses a complicated algorithm
//! - [leaky-bucket](https://crates.io/crates/leaky-bucket)
//!   - Async tasks can wait for their turn to use a resource.
//!   - Unsuitable for load shedding because there is no `try_acquire`.
//!
//! ## Related Crates
//! - [safe-dns](https://crates.io/crates/safe-dns) uses this
//!
//! ## Example
//! ```
//! # use prob_rate_limiter::ProbRateLimiter;
//! # use std::time::{Duration, Instant};
//! let mut limiter = ProbRateLimiter::new(10.0).unwrap();
//! let mut now = Instant::now();
//! assert!(limiter.check(5, now));
//! assert!(limiter.check(5, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(5, now));
//! assert!(limiter.check(5, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(5, now));
//! assert!(limiter.check(5, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(5, now));
//! assert!(limiter.check(5, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(5, now));
//! assert!(limiter.check(5, now));
//! assert!(!limiter.check(5, now));
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.0 - Initial version
//!
//! # TO DO
//! - Compare performance with `governor`
//! - Publish
#![forbid(unsafe_code)]

use core::time::Duration;
use oorandom::Rand32;
use std::time::Instant;

trait SaturatingAddAssign<T> {
    fn saturating_add_assign(&mut self, rhs: T);
}
impl SaturatingAddAssign<u32> for u32 {
    fn saturating_add_assign(&mut self, rhs: u32) {
        *self = self.saturating_add(rhs);
    }
}

fn decide(recent_cost: u32, max_cost: u32, mut rand_float: impl FnMut() -> f32) -> bool {
    // Value is in [0.0, 1.0).
    let load = if max_cost == 0 || recent_cost >= max_cost {
        return false;
    } else {
        f64::from(recent_cost) / f64::from(max_cost)
    };
    // Value is in (-inf, 1.0).
    let linear_reject_prob = (load - 0.75) * 4.0;
    if linear_reject_prob <= 0.0 {
        return true;
    }
    let reject_prob = linear_reject_prob.powi(2);
    reject_prob < rand_float().into()
}

#[cfg(test)]
#[test]
fn test_decide() {
    assert!(!decide(0, 0, || unreachable!()));
    assert!(decide(0, 100, || unreachable!()));
    assert!(decide(50, 100, || unreachable!()));
    assert!(decide(75, 100, || unreachable!()));
    assert!(decide(76, 100, || 0.999999));
    assert!(!decide(76, 100, || 0.0));
    assert!(!decide(85, 100, || 0.15));
    assert!(decide(85, 100, || 0.17));
    assert!(!decide(90, 100, || 0.35));
    assert!(decide(90, 100, || 0.37));
    assert!(!decide(95, 100, || 0.63));
    assert!(decide(95, 100, || 0.65));
    assert!(!decide(99, 100, || 0.92));
    assert!(decide(99, 100, || 0.93));
    assert!(!decide(100, 100, || unreachable!()));
    assert!(!decide(101, 100, || unreachable!()));
}

/// A probabilistic rate-limiter.
/// - When not overloaded, accepts all requests
/// - As load approaches limit, probabilistically rejects more and more requests.
/// - Onset of overload does not trigger a sudden total outage.
#[derive(Clone, Debug)]
pub struct ProbRateLimiter {
    tick_duration: Duration,
    max_cost: u32,
    cost: u32,
    last: Instant,
    prng: Rand32,
}
impl ProbRateLimiter {
    /// Makes a new rate limiter that accepts `max_cost_per_tick` every `tick_duration`.
    ///
    /// # Errors
    /// Returns an error when `tick_duration` is less than 1 microsecond.
    pub fn new_custom(
        tick_duration: Duration,
        max_cost_per_tick: u32,
        now: Instant,
        prng: Rand32,
    ) -> Result<Self, String> {
        if tick_duration.as_micros() == 0 {
            return Err(format!("tick_duration too small: {:?}", tick_duration));
        }
        Ok(Self {
            tick_duration,
            max_cost: max_cost_per_tick * 2,
            cost: 0_u32,
            last: now,
            prng,
        })
    }

    /// # Errors
    /// Returns an error when `max_cost_per_sec` is less than 1.0 or not finite.
    pub fn new(max_cost_per_sec: f32) -> Result<Self, String> {
        if !max_cost_per_sec.is_finite() || !max_cost_per_sec.is_sign_positive() {
            return Err(format!(
                "max_cost_per_sec is negative or not finite: {:?}",
                max_cost_per_sec
            ));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let max_cost_per_tick = max_cost_per_sec as u32;
        // TODO: Multiply all costs to support costs under 1.0 and to increase memory.
        if max_cost_per_sec != 0.0 && max_cost_per_tick == 0 {
            return Err(format!(
                "max_cost_per_sec too small: {:?}",
                max_cost_per_sec
            ));
        }
        Self::new_custom(
            Duration::from_secs(1),
            max_cost_per_tick,
            Instant::now(),
            Rand32::new(0),
        )
    }

    pub fn check(&mut self, cost: u32, now: Instant) -> bool {
        if self.max_cost == 0 {
            return false;
        }
        let elapsed = now.saturating_duration_since(self.last);
        #[allow(clippy::cast_possible_truncation)]
        let elapsed_ticks = (elapsed.as_micros() / self.tick_duration.as_micros()) as u32;
        self.last += self.tick_duration * elapsed_ticks;
        self.cost = self.cost.wrapping_shr(elapsed_ticks);
        if !decide(self.cost, self.max_cost, || self.prng.rand_float()) {
            return false;
        }
        self.cost.saturating_add_assign(cost);
        true
    }
}

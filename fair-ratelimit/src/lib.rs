//! [![crates.io version](https://img.shields.io/crates/v/fair-ratelimit.svg)](https://crates.io/crates/fair-ratelimit)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/fair-ratelimit/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # fair-ratelimit
//!
//! Use `RateLimiter` struct to detect overload and
//! fairly shed load from diverse users, systems, or IP addresses.
//! Prevent denial-of-service (`DoS`) attacks.
//!
//! ## Use Cases
//! - DNS server: DNS servers must send UDP replies without a handshake.
//!   Your DNS server could be used in a
//!   [DNS amplification attacks](https://www.cisa.gov/uscert/ncas/alerts/TA13-088A).
//!   Use this crate to prevent that.
//! - Server without handshake: If your server sends large responses without a handshake,
//!   it could be used in an amplification attack.  Use this crate to prevent that.
//! - Load balancer: Use this crate in a load balancer to avoid forwarding DoS attacks to
//!   backend systems.
//! - API server: Shed load from misbehaving clients
//!   and keep the API available for other clients.
//!
//! ## Features
//! - Global throughput limit
//! - IPv4 & IPv6
//! - `forbid(unsafe_code)`, depends only on crates that are `forbid(unsafe_code)`
//! - ?% test coverage
//!
//! ## Limitations
//!
//! ## Alternatives
//! - [governor](https://crates.io/crates/governor)
//!   - Popular
//!   - Lots of features
//!   - Good docs
//!   - Unnecessary `unsafe`
//!   - Uses non-standard mutex library [`parking_lot`](https://crates.io/crates/parking_lot)
//! - [r8limit](https://crates.io/crates/r8limit)
//!   - Simple
//!   - No `unsafe` or deps
//! - [leaky-bucket](https://crates.io/crates/leaky-bucket)
//!   - Async tasks can wait for their turn to use a resource.
//!   - Unsuitable for load shedding.
//!
//! ## Related Crates
//! - [safe-dns](https://crates.io/crates/safe-dns) uses this
//!
//! ## Example
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.0 - Initial version
//!
//! # TO DO
//! - Tests
//! - Implement
//! - Publish
//! - Example with subnet keys
//! - Example with IP keys
//! - Example with string keys
//! - Simulate bursty traffic
#![forbid(unsafe_code)]

mod ip_subnet;

use core::time::Duration;
use oorandom::Rand32;
use std::collections::HashMap;
use std::time::Instant;

pub use ip_subnet::IpSubnet;

const HORIZON_DURATION: Duration = Duration::from_secs(10);
const TICKS: usize = 10;
const TICK_DURATION: Duration =
    Duration::from_millis((HORIZON_DURATION.as_millis() / (TICKS as u128)) as u64);
const MAX_KEYS: usize = 100;

fn right_shift<T: Clone + Default>(slice: &mut [T], n: usize) {
    if n == 0 {
        return;
    }
    let n = n.min(slice.len());
    slice.rotate_right(n);
    slice[0..n].fill(Default::default());
}

#[cfg(test)]
#[test]
fn test_right_shift() {
    fn check(mut input: &mut [u8], n: usize, expected_output: &[u8]) {
        right_shift(&mut input, n);
        assert_eq!(expected_output, input);
    }
    check(&mut [], 0, &[]);
    check(&mut [], 1, &[]);
    check(&mut [], 11, &[]);
    check(&mut [1], 0, &[1]);
    check(&mut [1], 1, &[0]);
    check(&mut [1], 11, &[0]);
    check(&mut [1, 2], 0, &[1, 2]);
    check(&mut [1, 2], 1, &[0, 1]);
    check(&mut [1, 2], 11, &[0, 0]);
    check(&mut [1, 2, 3], 0, &[1, 2, 3]);
    check(&mut [1, 2, 3], 1, &[0, 1, 2]);
    check(&mut [1, 2, 3], 11, &[0, 0, 0]);
    check(&mut [1, 2, 3, 4, 5], 0, &[1, 2, 3, 4, 5]);
    check(&mut [1, 2, 3, 4, 5], 1, &[0, 1, 2, 3, 4]);
    check(&mut [1, 2, 3, 4, 5], 3, &[0, 0, 0, 1, 2]);
}

trait SaturatingAddAssign<T> {
    fn saturating_add_assign(&mut self, rhs: T);
}
impl SaturatingAddAssign<u32> for u32 {
    fn saturating_add_assign(&mut self, rhs: u32) {
        *self = self.saturating_add(rhs)
    }
}

fn decide(recent_cost: u32, max_cost: u32, mut rand_float: impl FnMut() -> f32) -> bool {
    // Value is in [0.0, 1.0).
    let load = if max_cost == 0 || recent_cost >= max_cost {
        return false;
    } else {
        (recent_cost as f32) / (max_cost as f32)
    };
    // Value is in (-inf, 1.0).
    let linear_reject_prob = (load - 0.75) * 4.0;
    if linear_reject_prob <= 0.0 {
        return true;
    }
    let reject_prob = linear_reject_prob.powi(2);
    reject_prob < rand_float()
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

/// When recent load is in (0.75,1.0], linearly interpolate max cost between
/// global_max_cost and global_max_cost/keys.
fn max_cost(sources_max: u32, recent_cost: u32, keys: u32) -> u32 {
    if sources_max < 1 {
        return 0;
    }
    let load = (recent_cost as f32) / (sources_max as f32);
    if keys < 1 {
        sources_max
    } else if load > 1.0 {
        ((sources_max as f32) / (keys as f32)) as u32
    } else if load > 0.75 {
        let x = (load - 0.75) * 4.0;
        // f(x) = ax + b
        // f(0.0) = global_max_cost = b
        // f(1.0) = global_max_cost/keys
        // f(x) = -(global_max_cost - global_max_cost/keys)x + global_max_cost
        // f(x) = global_max_cost - (global_max_cost - global_max_cost/keys)x
        // f(x) = global_max_cost(1 - (1 - 1/keys)x)
        ((sources_max as f32) * (1.0 - (1.0 - 1.0 / (keys as f32)) * x)) as u32
    } else {
        sources_max
    }
}

#[cfg(test)]
#[test]
fn test_max_cost() {
    assert_eq!(100, max_cost(100, 0, 0));
    assert_eq!(100, max_cost(100, 0, 1));
    assert_eq!(100, max_cost(100, 1, 1));
    assert_eq!(100, max_cost(100, 100, 1));

    assert_eq!(100, max_cost(100, 0, 2));
    assert_eq!(100, max_cost(100, 75, 2));
    assert_eq!(98, max_cost(100, 76, 2));
    assert_eq!(70, max_cost(100, 90, 2));
    assert_eq!(52, max_cost(100, 99, 2));
    assert_eq!(50, max_cost(100, 100, 2));
    assert_eq!(50, max_cost(100, 150, 2));

    assert_eq!(1000, max_cost(1000, 0, 10));
    assert_eq!(1000, max_cost(1000, 750, 10));
    assert_eq!(996, max_cost(1000, 751, 10));
    assert_eq!(460, max_cost(1000, 900, 10));
    assert_eq!(103, max_cost(1000, 999, 10));
    assert_eq!(100, max_cost(1000, 1000, 10));
    assert_eq!(100, max_cost(1000, 1500, 10));
}

#[derive(Clone, Copy, Debug)]
struct RecentCosts {
    costs: [u32; TICKS],
    last: Instant,
}
impl RecentCosts {
    #[must_use]
    pub fn new(now: Instant) -> Self {
        Self {
            costs: [0_u32; TICKS],
            last: now,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.costs.iter().all(|elem| *elem == 0)
    }

    pub fn add(&mut self, cost: u32) {
        self.costs[0].saturating_add_assign(cost);
    }

    pub fn update(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last);
        let elapsed_ticks = (elapsed.as_millis() / TICK_DURATION.as_millis()) as u32;
        self.last = self.last + (TICK_DURATION * elapsed_ticks);
        right_shift(&mut self.costs, elapsed_ticks as usize);
    }

    #[must_use]
    pub fn recent_cost(&self) -> u32 {
        self.costs
            .iter()
            .fold(0_u32, |acc, elem| acc.saturating_add(*elem))
    }
}

#[derive(Clone, Copy, Debug)]
struct Source {
    pub key: u32,
    pub costs: RecentCosts,
}
impl Source {
    pub fn new(key: u32, now: Instant) -> Self {
        Self {
            key,
            costs: RecentCosts::new(now),
        }
    }
}

/// Features:
/// - Probabilistically rejects requests.
///   Normal overload causes an increase in latency as clients retry.
///   Overload does not trigger a sudden total outage for any group of users.
/// - In overload, try to give every IP address the same throughput.
///   As recent load approaches overload, gradually increase fairness.
///   A limited source of overload will get throttled and leave other traffic untouched.
///
/// Implementation:
/// - Keep a map of target IPs to the count of bytes sent recently.
/// - When adding a new IP to a full map, multiply the count of bytes by a random coefficient.
///   Create the coefficient based on the size of the smallest count, so the smallest count has
///   a 50% chance of getting replaced by a new packet of the same size.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    sources_max: u32,
    other_max: u32,
    prng: Rand32,
    sources_costs: RecentCosts,
    keys: HashMap<u32, usize>,
    sources: [Option<Source>; MAX_KEYS],
    other_costs: RecentCosts,
}
impl RateLimiter {
    #[must_use]
    pub fn new(max_cost_per_sec: u32, prng: Rand32, now: Instant) -> Self {
        // TODO: Ensure that values are not too small.
        let global_max =
            (((max_cost_per_sec as u128) * HORIZON_DURATION.as_millis()) / 1_000) as f32;
        let sources_max = (global_max * 0.80) as u32;
        let other_max = (global_max * 0.20) as u32;
        Self {
            sources_max,
            other_max,
            prng,
            sources_costs: RecentCosts::new(now),
            keys: HashMap::with_capacity(MAX_KEYS),
            sources: [None; MAX_KEYS],
            other_costs: RecentCosts::new(now),
        }
    }

    fn update(&mut self, key: u32, now: Instant) {
        if let Some(index) = self.keys.get(&key) {
            let source = self.sources[*index].as_mut().unwrap();
            source.costs.update(now);
            if source.costs.is_empty() {
                self.sources[*index] = None;
                self.keys.remove(&key);
            }
        }
    }

    fn add(&mut self, key: u32, cost: u32, now: Instant) {
        if let Some(index) = self.keys.get(&key) {
            self.sources[*index].as_mut().unwrap().costs.add(cost);
            return;
        }
        // Source is unknown.  Try to add it.
        let index = self.prng.rand_range(0..(MAX_KEYS as u32)) as usize;
        if let Some(source) = &mut self.sources[index] {
            source.costs.update(now);
            // With a small probability, multiply cost by a large coefficient.
            // This lets a busy source eventually get a spot in a full table.
            let coefficient: u32 = match self.prng.rand_range(0..10_000u32) {
                0 => 10_000,
                x if x < 10 => 1_000,
                x if x < 100 => 100,
                x if x < 1000 => 10,
                _ => 1,
            };
            let adjusted_cost = coefficient.saturating_mul(cost);
            if adjusted_cost < source.costs.recent_cost() {
                // Do not evict entry.  This source will remain unknown.
                self.other_costs.add(cost);
                return;
            }
            // Evict entry.
            self.keys.remove(&source.key);
        }
        // Remember source.
        self.keys.insert(key, index);
        let mut new_source = Source::new(key, now);
        new_source.costs.add(cost);
        self.sources[index] = Some(new_source);
    }

    pub fn check(&mut self, key: u32, cost: u32, now: Instant) -> bool {
        self.sources_costs.update(now);
        self.update(key, now);
        let (recent_cost, max_cost) = if let Some(index) = self.keys.get(&key) {
            let recent_cost = self.sources[*index].as_ref().unwrap().costs.recent_cost();
            let max_cost = max_cost(
                self.sources_max,
                self.sources_costs.recent_cost(),
                self.keys.len() as u32,
            );
            (recent_cost, max_cost)
        } else {
            self.other_costs.update(now);
            (self.other_costs.recent_cost(), self.other_max)
        };
        if decide(recent_cost, max_cost, || self.prng.rand_float()) {
            self.sources_costs.add(cost);
            self.add(key, cost, now);
            true
        } else {
            false
        }
    }
}

//! fair-rate-limiter
//! =================
//! [![crates.io version](https://img.shields.io/crates/v/fair-rate-limiter.svg)](https://crates.io/crates/fair-rate-limiter)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/fair-rate-limiter/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! Use `RateLimiter` struct to detect overload and
//! fairly shed load from diverse IP addresses, users, or systems.
//! Mitigate denial-of-service (`DoS`) attacks.
//!
//! # Use Cases
//! - DNS server: DNS servers must send UDP replies without a handshake.
//!   Your DNS server could be used in a
//!   [DNS amplification attacks](https://www.cisa.gov/uscert/ncas/alerts/TA13-088A).
//!   Use this crate to prevent that.
//! - Server without handshake: If your server sends large responses without a handshake,
//!   it could be used in an amplification attack.  Use this crate to prevent that.
//! - Load balancer: Use this crate in a load balancer to avoid forwarding `DoS` attacks to
//!   backend systems.
//! - API server: Shed load from misbehaving clients
//!   and keep the API available for other clients.
//!
//! # Features
//! - Global throughput limit
//! - IPv4 & IPv6
//! - `forbid(unsafe_code)`, depends only on crates that are `forbid(unsafe_code)`
//! - 83% test coverage
//! - Optimized.  Performance on an i5-8259U:
//!   - Internal service tracking 10 clients: 150ns per check, 7M checks per second
//!   - Public service tracking 1M clients: 500ns per check, 2M checks per second
//!   - `DDoS` mitigation tracking 30M clients: 750ns per check, 1.3M checks per second
//!
//! # Limitations
//!
//! # Alternatives
//! - [governor](https://crates.io/crates/governor)
//!   - Popular
//!   - Lots of features
//!   - Good docs
//!   - Unnecessary `unsafe`
//!   - Uses non-standard mutex library [`parking_lot`](https://crates.io/crates/parking_lot)
//! - [r8limit](https://crates.io/crates/r8limit)
//!   - Supports a single bucket.  Usable for unfair load shedding.
//!   - No `unsafe` or deps
//! - [leaky-bucket](https://crates.io/crates/leaky-bucket)
//!   - Async tasks can wait for their turn to use a resource.
//!   - Unsuitable for load shedding.
//!
//! # Related Crates
//! - [safe-dns](https://crates.io/crates/safe-dns) uses this
//!
//! # Example
//! ```
//! # use fair_rate_limiter::{new_fair_ip_address_rate_limiter, IpAddrKey, FairRateLimiter};
//! # use oorandom::Rand32;
//! # use std::net::Ipv4Addr;
//! # use std::time::{Duration, Instant};
//! let mut limiter = new_fair_ip_address_rate_limiter(10.0).unwrap();
//! let mut now = Instant::now();
//! let key = IpAddrKey::from(Ipv4Addr::new(10,0,0,1));
//! assert!(limiter.check(key, 4, now));
//! assert!(limiter.check(key, 4, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(key, 4, now));
//! assert!(limiter.check(key, 4, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(key, 4, now));
//! assert!(limiter.check(key, 4, now));
//! now += Duration::from_secs(1);
//! assert!(limiter.check(key, 4, now));
//! assert!(limiter.check(key, 4, now));
//! assert!(!limiter.check(key, 4, now));
//! ```
//!
//! # Cargo Geiger Safety Report
//! # Changelog
//! - v0.1.0 - Initial version
//!
//! # TO DO
//! - Rename to `fair-rate-limiter`
//! - Compare performance with `governor`
//! - Publish
//! - Simulate bursty traffic
//! - Measure memory consumption, add to Limitations section
//! - Replace hash table with skip list and see if performance improves
//! - Support concurrent use
//! - Allow tracked sources to use unused untracked throughput allocation
//! - Adjust `tick_duration` to support `max_cost_per_sec` < 1.0
#![forbid(unsafe_code)]

use core::time::Duration;
use oorandom::Rand32;
use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Instant;

/// Copied from unstable `std::net::ip::Ipv6Addr::to_ipv4_mapped`.
#[must_use]
const fn to_ipv4_mapped(addr: &Ipv6Addr) -> Option<Ipv4Addr> {
    match addr.octets() {
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, a, b, c, d] => Some(Ipv4Addr::new(a, b, c, d)),
        _ => None,
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IpAddrKey(Ipv6Addr);
impl IpAddrKey {
    #[must_use]
    pub fn new(ip_addr: IpAddr) -> Self {
        match ip_addr {
            IpAddr::V4(addr) => Self(addr.to_ipv6_mapped()),
            IpAddr::V6(addr) => Self(addr),
        }
    }
}
impl From<IpAddr> for IpAddrKey {
    fn from(addr: IpAddr) -> Self {
        Self::new(addr)
    }
}
impl From<Ipv4Addr> for IpAddrKey {
    fn from(addr: Ipv4Addr) -> Self {
        Self(addr.to_ipv6_mapped())
    }
}
impl From<Ipv6Addr> for IpAddrKey {
    fn from(addr: Ipv6Addr) -> Self {
        Self(addr)
    }
}
impl Display for IpAddrKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(addr) = to_ipv4_mapped(&self.0) {
            write!(f, "{}", addr)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

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
#[allow(clippy::unreadable_literal)]
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
/// `global_max_cost` and `global_max_cost`/keys.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn max_cost(sources_max: u32, recent_cost: u32, keys: u32) -> u32 {
    if sources_max < 1 {
        return 0;
    }
    let load = f64::from(recent_cost) / f64::from(sources_max);
    if keys < 1 {
        sources_max
    } else if load > 1.0 {
        (f64::from(sources_max) / f64::from(keys)) as u32
    } else if load > 0.75 {
        let x = (load - 0.75) * 4.0;
        // f(x) = ax + b
        // f(0.0) = global_max_cost = b
        // f(1.0) = global_max_cost/keys
        // f(x) = -(global_max_cost - global_max_cost/keys)x + global_max_cost
        // f(x) = global_max_cost - (global_max_cost - global_max_cost/keys)x
        // f(x) = global_max_cost(1 - (1 - 1/keys)x)
        (f64::from(sources_max) * (1.0 - (1.0 - 1.0 / f64::from(keys)) * x)) as u32
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
    assert_eq!(459, max_cost(1000, 900, 10));
    assert_eq!(103, max_cost(1000, 999, 10));
    assert_eq!(99, max_cost(1000, 1000, 10));
    assert_eq!(100, max_cost(1000, 1500, 10));
}

#[derive(Clone, Copy, Debug)]
struct RecentCosts {
    cost: u32,
    last: Instant,
}
impl RecentCosts {
    #[must_use]
    pub fn new(now: Instant) -> Self {
        Self {
            cost: 0_u32,
            last: now,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cost == 0
    }

    pub fn add(&mut self, cost: u32) {
        self.cost.saturating_add_assign(cost);
    }

    pub fn update(&mut self, tick_duration: Duration, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last);
        #[allow(clippy::cast_possible_truncation)]
        let elapsed_ticks = (elapsed.as_millis() / tick_duration.as_millis()) as u32;
        self.last += tick_duration * elapsed_ticks;
        self.cost = self.cost.wrapping_shr(elapsed_ticks);
    }

    #[must_use]
    pub fn recent_cost(&self) -> u32 {
        self.cost
    }
}

#[derive(Clone, Copy, Debug)]
struct Source<K> {
    pub key: K,
    pub costs: RecentCosts,
}
impl<K> Source<K> {
    pub fn new(key: K, now: Instant) -> Self {
        Self {
            key,
            costs: RecentCosts::new(now),
        }
    }
}

/// A fair rate-limiter.
/// - Probabilistically rejects requests.
/// - When not overloaded, allows any source to freely consume throughput.
/// - Gradually increases fairness as load approaches overload.
/// - Onset of overload does not trigger a sudden total outage for any group of users.
/// - In overload, it tries to give every source the same throughput.
/// - A limited source of overload will get throttled and leave other traffic untouched.
///
/// Can track `MaxKeys` sources.
///
/// Each source has a key with type `K`.
/// [`IpAddrKey`](struct.IpAddrKey.html) is useful for this.
#[derive(Clone, Debug)]
pub struct FairRateLimiter<K: Clone + Copy + Eq + Hash, const MAX_KEYS: usize> {
    tick_duration: Duration,
    sources_max: u32,
    other_max: u32,
    prng: Rand32,
    sources_costs: RecentCosts,
    keys: HashMap<K, usize>,
    sources: Box<[Option<Source<K>>]>,
    other_costs: RecentCosts,
}
impl<Key: Clone + Copy + Eq + Hash, const MAX_KEYS: usize> FairRateLimiter<Key, MAX_KEYS> {
    /// Make a new rate limiter.
    ///
    /// The rate limiter remembers 100 clients which recently had the heaviest load.
    /// It allows `tracked_max_cost_per_sec` traffic from these clients.
    ///
    /// Other clients with lighter load are untracked.
    /// If you make a bar graph of clients, ordered by their traffic, these are the long tail.
    /// The rate limiter allows `other_max_cost_per_sec` traffic from these clients.
    ///
    /// Set both numbers to zero to stop all requests.
    ///
    /// # Errors
    /// Returns error when `tick_duration` is less than a 1 microsecond.
    pub fn new(
        tick_duration: Duration,
        max_cost_per_tick_from_tracked_sources: u32,
        max_cost_per_tick_from_untracked_sources: u32,
        prng: Rand32,
        now: Instant,
    ) -> Result<Self, String> {
        if tick_duration.as_micros() == 0 {
            return Err(format!("tick_duration too small: {:?}", tick_duration));
        }
        Ok(Self {
            tick_duration,
            sources_max: max_cost_per_tick_from_tracked_sources * 2,
            other_max: max_cost_per_tick_from_untracked_sources * 2,
            prng,
            sources_costs: RecentCosts::new(now),
            keys: HashMap::with_capacity(MAX_KEYS),
            sources: vec![None; MAX_KEYS].into_boxed_slice(),
            other_costs: RecentCosts::new(now),
        })
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn check(&mut self, key: Key, cost: u32, now: Instant) -> bool {
        self.sources_costs.update(self.tick_duration, now);
        #[allow(clippy::cast_possible_truncation)]
        let num_keys = self.keys.len() as u32;
        match self.keys.entry(key) {
            Entry::Occupied(entry) => {
                // The key was found.
                // Decide whether to reject or accept the request.
                let index = *entry.get();
                let source = self.sources[index].as_mut().unwrap();
                source.costs.update(self.tick_duration, now);
                let max_cost =
                    max_cost(self.sources_max, self.sources_costs.recent_cost(), num_keys);
                let rand_float = || self.prng.rand_float();
                if decide(source.costs.recent_cost(), max_cost, rand_float) {
                    // Accept the request.
                    self.sources_costs.add(cost);
                    source.costs.add(cost);
                    true
                } else {
                    // Reject the request.
                    if source.costs.is_empty() {
                        // The key has no recent costs.  Discard it.
                        entry.remove();
                        self.sources[index] = None;
                    }
                    false
                }
            }
            Entry::Vacant(entry) => {
                // The key was not found.
                // Decide whether to reject the request right away.
                self.other_costs.update(self.tick_duration, now);
                let recent_cost = self.other_costs.recent_cost();
                if !decide(recent_cost, self.other_max, || self.prng.rand_float()) {
                    return false;
                }
                // Pick a random slot.
                let mut new_source = Source::new(*entry.key(), now);
                new_source.costs.add(cost);
                #[allow(clippy::cast_possible_truncation)]
                let index = self.prng.rand_range(0..(MAX_KEYS as u32)) as usize;
                if let Some(source) = &mut self.sources[index] {
                    // Slot is used.  Decide whether or not to replace it.
                    source.costs.update(self.tick_duration, now);
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
                        // Do not replace entry.  This source will remain unknown.
                        self.other_costs.add(cost);
                        true
                    } else {
                        // Replace entry.
                        entry.insert(index);
                        self.keys.remove(&source.key);
                        *source = new_source;
                        self.sources_costs.add(cost);
                        true
                    }
                } else {
                    // Slot is unused.
                    self.sources[index] = Some(new_source);
                    entry.insert(index);
                    self.sources_costs.add(cost);
                    true
                }
            }
        }
    }
}

/// Creates a new fair rate limiter for IP addresses.
///
/// # Errors
/// Returns an error when `max_cost_per_sec` is less than 1.0.
pub fn new_fair_ip_address_rate_limiter(
    max_cost_per_sec: f32,
) -> Result<FairRateLimiter<IpAddrKey, 1000>, String> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let other_max = max((max_cost_per_sec * 0.20) as u32, 1);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let sources_max = (max_cost_per_sec as u32).saturating_sub(other_max);
    if max_cost_per_sec != 0.0 && sources_max == 0 {
        return Err(format!(
            "max_cost_per_sec is too small: {:?}",
            max_cost_per_sec
        ));
    }
    FairRateLimiter::new(
        Duration::from_secs(1),
        sources_max,
        other_max,
        Rand32::new(0),
        Instant::now(),
    )
}

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
const TICK_DURATION: Duration = Duration::from_millis((HORIZON_DURATION.as_millis() / 10) as u64);

fn right_shift<T: Clone + Default>(slice: &mut [T], n: usize) {
    if n == 0 {
        return;
    }
    let n = n.min(slice.len());
    slice.rotate_right(n);
    slice[0..n].fill(Default::default());
}

fn saturating_f32_to_usize(x: f32) -> usize {
    if x == f32::NAN {
        0
    } else if x < 0.0 {
        0
    } else if x > (usize::MAX as f32) {
        usize::MAX
    } else {
        x as usize
    }
}

fn saturating_f32_to_u32(x: f32) -> u32 {
    if x == f32::NAN {
        0
    } else if x < 0.0 {
        0
    } else if x > (u32::MAX as f32) {
        u32::MAX
    } else {
        x as u32
    }
}

trait SaturatingAddAssign<T> {
    fn saturating_add_assign(&mut self, rhs: T);
}
impl SaturatingAddAssign<u32> for u32 {
    fn saturating_add_assign(&mut self, rhs: u32) {
        *self = self.saturating_add(rhs)
    }
}

/// Features:
/// - Probabilistically rejects requests.
///   Normal overload causes an increase in latency as clients retry.
///   Overload does not trigger a sudden total outage for any group of users.
/// - In overload, try to give every IP address the same throughput.
///
/// Implementation:
/// - Keep a map of target IPs to the count of bytes sent recently.
/// - When adding a new IP to a full map, multiply the count of bytes by a random coefficient.
///   Create the coefficient based on the size of the smallest count, so the smallest count has
///   a 50% chance of getting replaced by a new packet of the same size.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    max_cost: u32,
    last: Instant,
    prng: Rand32,
    global_costs: [u32; 10],
    source_costs: HashMap<u32, [u32; 10]>,
}
impl RateLimiter {
    #[must_use]
    pub fn new(max_cost_per_sec: u32, prng: Rand32, now: Instant) -> Self {
        Self {
            max_cost: (((max_cost_per_sec as u128) * HORIZON_DURATION.as_millis()) / 1_000) as u32,
            last: now,
            prng,
            global_costs: [0_u32; 10],
            source_costs: HashMap::new(),
        }
    }

    pub fn check(&mut self, key: u32, cost: u32, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last);
        let elapsed_ticks = (elapsed.as_millis() / TICK_DURATION.as_millis()) as u32;
        self.last = self.last + (TICK_DURATION * elapsed_ticks);
        right_shift(&mut self.global_costs, elapsed_ticks as usize);
        let recent_cost = self
            .global_costs
            .iter()
            .fold(0_u32, |acc, elem| acc.saturating_add(*elem));
        if self.max_cost == 0 {
            return false;
        }
        let recent_load = (recent_cost as f32) / (self.max_cost as f32);
        let reject_prob = (recent_load - 0.75) * 4.0;
        if reject_prob > 0.0 && self.prng.rand_float() < reject_prob {
            return false;
        }
        self.global_costs[0].saturating_add_assign(cost);
        true
    }
}

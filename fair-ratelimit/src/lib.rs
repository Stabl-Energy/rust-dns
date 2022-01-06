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

use oorandom::Rand32;
use std::time::Instant;

pub use ip_subnet::IpSubnet;

/// Features:
/// - Probabilistically rejects requests.
///   Normal overload causes an increase in latency as clients retry.
///   Overload does not trigger a sudden total outage for any group of users.
/// - In overload, try to give every IP address the same throughput.
///
/// Implementation:
/// - Keep a map of target IPs to the count of bytes sent recently.
/// - Every minute, multiply the counts by a coefficient less than 1.0.
/// - When adding a new IP to a full map, multiply the count of bytes by a random coefficient.
///   Create the coefficient based on the size of the smallest count, so the smallest count has
///   a 50% chance of getting replaced by a new packet of the same size.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    max_cost_per_sec: u32,
    prng: Rand32,
}
impl RateLimiter {
    #[must_use]
    pub fn new(max_cost_per_sec: u32, prng: Rand32) -> Self {
        Self {
            max_cost_per_sec,
            prng,
        }
    }

    pub fn check(&mut self, key: u32, cost: u32, now: Instant) -> bool {
        true
    }
}

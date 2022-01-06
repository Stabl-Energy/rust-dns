use core::cell::Cell;
use core::cmp::Ordering;
use core::time::Duration;
use fair_ratelimit::RateLimiter;
use oorandom::Rand32;
use std::collections::BinaryHeap;
use std::time::Instant;

trait Client {
    fn check(&mut self, limiter: &mut RateLimiter, now: Instant) -> Instant;
}

enum Key {
    Static(u32),
    Random(Cell<Rand32>),
}
impl Key {
    pub fn get(&mut self) -> u32 {
        match self {
            Key::Static(value) => *value,
            Key::Random(rand32_cell) => rand32_cell.get_mut().rand_u32(),
        }
    }
}

struct SteadyClient {
    key: Key,
    interval: Duration,
    cost: u32,
    pub accepted_requests: u32,
    pub rejected_request: u32,
}
impl SteadyClient {
    pub fn new(key: Key, rps: u32, cost: u32) -> Self {
        Self {
            key,
            interval: Duration::from_secs(1) / rps,
            cost,
            accepted_requests: 0,
            rejected_request: 0,
        }
    }
}
impl Client for SteadyClient {
    fn check(&mut self, limiter: &mut RateLimiter, now: Instant) -> Instant {
        if limiter.check(self.key.get(), self.cost, now) {
            self.accepted_requests += 1;
        } else {
            self.rejected_request += 1;
        }
        now + self.interval
    }
}

fn simulate<'x>(
    num_seconds: u64,
    limiter: &mut RateLimiter,
    clients: &'x mut [&'x mut dyn Client],
) {
    struct Entry<'y>(Instant, &'y mut dyn Client);
    impl<'y> Eq for Entry<'y> {}
    impl<'y> PartialEq<Self> for Entry<'y> {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }
    impl<'y> PartialOrd<Self> for Entry<'y> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl<'y> Ord for Entry<'y> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0).reverse()
        }
    }
    let mut now = Instant::now();
    let deadline = now + Duration::from_secs(num_seconds);
    let mut heap: BinaryHeap<Entry<'x>> = clients
        .into_iter()
        .map(|client| Entry(now, *client))
        .collect();
    loop {
        let entry = heap.pop().unwrap();
        let (new_now, client): (Instant, &'x mut dyn Client) = (entry.0, entry.1);
        assert!(now <= new_now);
        now = new_now;
        if deadline <= now {
            break;
        }
        let next_request_instant = client.check(limiter, now);
        heap.push(Entry(next_request_instant, client));
    }
}

macro_rules! assert_contains {
    ( $range:expr, $value:expr ) => {
        if !$range.contains(&$value) {
            panic!("{:?} is not in {:?}", $value, $range);
        }
    };
}

#[test]
fn test_single_request() {
    let mut limiter = RateLimiter::new(1, Rand32::new(1));
    assert!(limiter.check(0, 1, Instant::now()));
}

#[test]
fn test_single_client() {
    let mut limiter = RateLimiter::new(2, Rand32::new(1));
    let mut client = SteadyClient::new(Key::Static(0), 1, 1);
    simulate(1000, &mut limiter, &mut [&mut client]);
    assert_eq!(1000, client.accepted_requests);
    assert_eq!(0, client.rejected_request);
}

#[test]
fn test_four_clients() {
    let mut limiter = RateLimiter::new(10, Rand32::new(1));
    let mut client0 = SteadyClient::new(Key::Static(0), 100, 1);
    let mut client1 = SteadyClient::new(Key::Static(1), 50, 1);
    let mut client2 = SteadyClient::new(Key::Static(2), 10, 1);
    let mut client3 = SteadyClient::new(Key::Static(3), 1, 1);
    simulate(
        1000,
        &mut limiter,
        &mut [&mut client0, &mut client1, &mut client2, &mut client3],
    );
    assert_contains!(2000..3000, client0.accepted_requests);
    assert_contains!(2000..3000, client1.accepted_requests);
    assert_contains!(2000..3000, client2.accepted_requests);
    assert_contains!(2000..3000, client3.accepted_requests);
    assert_contains!(
        9000..10_000,
        client0.accepted_requests
            + client1.accepted_requests
            + client2.accepted_requests
            + client3.accepted_requests
    );
}

#[test]
fn test_client_and_long_tail() {
    let mut limiter = RateLimiter::new(10, Rand32::new(1));
    let mut client = SteadyClient::new(Key::Static(0), 100, 1);
    let mut longtail = SteadyClient::new(Key::Random(Cell::new(Rand32::new(2))), 5, 1);
    simulate(1000, &mut limiter, &mut [&mut client, &mut longtail]);
    assert_contains!(4500..5500, client.accepted_requests);
    assert_contains!(4500..5500, longtail.accepted_requests);
    assert_contains!(
        9000..10_000,
        client.accepted_requests + longtail.accepted_requests
    );
}

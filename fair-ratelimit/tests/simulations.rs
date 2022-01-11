use core::cell::Cell;
use core::cmp::Ordering;
use core::time::Duration;
use fair_ratelimit::RateLimiter;
use oorandom::Rand32;
use std::cell::{Ref, RefCell};
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::time::Instant;

enum Key {
    Static(u32),
    Random(Cell<Rand32>),
}
impl Key {
    pub fn get(&mut self) -> u32 {
        match self {
            Key::Static(value) => *value,
            Key::Random(rand32_cell) => rand32_cell.get_mut().rand_range(1_000..2_000),
        }
    }
}
impl Debug for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Static(x) => write!(f, "Key::Static({})", x),
            Key::Random(_) => write!(f, "Key::Random"),
        }
    }
}

#[derive(Debug)]
struct Client {
    key: Key,
    interval: Duration,
    cost: u32,
    pub accepted_requests: u32,
    pub rejected_request: u32,
}
impl Client {
    pub fn new(key: Key, rps: u32, cost: u32) -> Self {
        Self {
            key,
            interval: Duration::from_secs(1) / rps,
            cost,
            accepted_requests: 0,
            rejected_request: 0,
        }
    }

    fn check(&mut self, limiter: &mut RateLimiter, now: Instant) -> Instant {
        if limiter.check(self.key.get(), self.cost, now) {
            self.accepted_requests += 1;
        } else {
            self.rejected_request += 1;
        }
        now + self.interval
    }
}

fn simulate(num_seconds: u64, limiter: &mut RateLimiter, clients: &[Rc<RefCell<Client>>]) {
    struct Entry(Instant, Rc<RefCell<Client>>);
    impl Eq for Entry {}
    impl PartialEq<Self> for Entry {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }
    impl PartialOrd<Self> for Entry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Entry {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0).reverse()
        }
    }
    let mut now = Instant::now();
    let deadline = now + Duration::from_secs(num_seconds);
    let mut heap: BinaryHeap<Entry> = clients.iter().map(|rc| Entry(now, Rc::clone(rc))).collect();
    let mut num_requests = 0;
    loop {
        let entry = heap.pop().unwrap();
        let (new_now, client): (Instant, Rc<RefCell<Client>>) = (entry.0, entry.1);
        assert!(now <= new_now);
        now = new_now;
        if deadline <= now {
            break;
        }
        let next_request_instant = client.borrow_mut().check(limiter, now);
        heap.push(Entry(next_request_instant, client));
        num_requests += 1;
    }
    println!(
        "Simulated {} request over {} seconds",
        num_requests, num_seconds
    );
    for rc_ref_cell_client in clients {
        let client: Ref<Client> = rc_ref_cell_client.borrow();
        println!("client: {:?}", client);
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
    let now = Instant::now();
    let mut limiter = RateLimiter::new(1, Rand32::new(1), now);
    assert!(limiter.check(0, 1, now));
}

#[test]
fn test_single_client() {
    let mut limiter = RateLimiter::new(2, Rand32::new(1), Instant::now());
    let client = Rc::new(RefCell::new(Client::new(Key::Static(0), 1, 1)));
    simulate(1000, &mut limiter, &[Rc::clone(&client)]);
    assert_eq!(1000, client.borrow().accepted_requests);
    assert_eq!(0, client.borrow().rejected_request);
}

#[test]
fn test_single_client_overload() {
    let mut limiter = RateLimiter::new(1, Rand32::new(1), Instant::now());
    let client = Rc::new(RefCell::new(Client::new(Key::Static(0), 1, 5)));
    simulate(1000, &mut limiter, &[Rc::clone(&client)]);
    assert_contains!(150..250, client.borrow().accepted_requests);
    assert_contains!(750..850, client.borrow().rejected_request);
}

#[test]
fn test_four_clients() {
    let mut limiter = RateLimiter::new(10, Rand32::new(1), Instant::now());
    let client0 = Rc::new(RefCell::new(Client::new(Key::Static(0), 100, 1)));
    let client1 = Rc::new(RefCell::new(Client::new(Key::Static(1), 50, 1)));
    let client2 = Rc::new(RefCell::new(Client::new(Key::Static(2), 10, 1)));
    let client3 = Rc::new(RefCell::new(Client::new(Key::Static(3), 1, 1)));
    simulate(
        1000,
        &mut limiter,
        &[
            Rc::clone(&client0),
            Rc::clone(&client1),
            Rc::clone(&client2),
            Rc::clone(&client3),
        ],
    );
    assert_contains!(2000..3000, client0.borrow().accepted_requests);
    assert_contains!(2000..3000, client1.borrow().accepted_requests);
    assert_contains!(2000..3000, client2.borrow().accepted_requests);
    assert_contains!(2000..3000, client3.borrow().accepted_requests);
    assert_contains!(
        9000..10_000,
        client0.borrow().accepted_requests
            + client1.borrow().accepted_requests
            + client2.borrow().accepted_requests
            + client3.borrow().accepted_requests
    );
}

#[test]
fn test_client_and_long_tail() {
    let mut limiter = RateLimiter::new(10, Rand32::new(1), Instant::now());
    let client = Rc::new(RefCell::new(Client::new(Key::Static(0), 5, 1)));
    let longtail = Rc::new(RefCell::new(Client::new(
        Key::Random(Cell::new(Rand32::new(2))),
        5,
        1,
    )));
    simulate(
        1000,
        &mut limiter,
        &[Rc::clone(&client), Rc::clone(&longtail)],
    );
    assert_contains!(4900..5000, client.borrow().accepted_requests);
    assert_contains!(4900..5000, longtail.borrow().accepted_requests);
}

use core::cell::Cell;
use core::cmp::Ordering;
use core::time::Duration;
use fair_rate_limiter::{FairRateLimiter, IpAddrKey};
use oorandom::Rand32;
use std::cell::{Ref, RefCell};
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone)]
enum Key {
    Static(u8),
    Random(Cell<Rand32>),
}
impl Key {
    pub fn get(&mut self) -> IpAddrKey {
        match self {
            Key::Static(value) => IpAddrKey::from(Ipv4Addr::new(10, 0, 0, *value)),
            Key::Random(rand32_cell) => {
                let rand32 = rand32_cell.get_mut();
                if rand32.rand_range(0..2) == 0 {
                    IpAddrKey::from(Ipv4Addr::from(rand32.rand_u32()))
                } else {
                    IpAddrKey::from(Ipv6Addr::new(
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                        (rand32.rand_u32() >> 16) as u16,
                    ))
                }
            }
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
    rps: u32,
    cost: u32,
    pub accepted_requests: u32,
}
impl Client {
    pub fn new(key: Key, rps: u32, cost: u32) -> Self {
        Self {
            key,
            rps,
            cost,
            accepted_requests: 0,
        }
    }

    fn check(&mut self, limiter: &mut FairRateLimiter<IpAddrKey, 100>, now: Instant) -> Instant {
        if limiter.check(self.key.get(), self.cost, now) {
            self.accepted_requests += 1;
        }
        now + (Duration::from_secs(1) / self.rps)
    }
}

fn simulate(
    limiter: &mut FairRateLimiter<IpAddrKey, 100>,
    clock: &mut Instant,
    num_seconds: u64,
    clients: &[&Rc<RefCell<Client>>],
) {
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
    let deadline = *clock + Duration::from_secs(num_seconds);
    let mut heap: BinaryHeap<Entry> = clients
        .iter()
        .map(|rc| Entry(*clock, Rc::clone(rc)))
        .collect();
    let mut num_requests = 0;
    loop {
        let entry = heap.pop().unwrap();
        let (new_now, client): (Instant, Rc<RefCell<Client>>) = (entry.0, entry.1);
        assert!(*clock <= new_now);
        *clock = new_now;
        if deadline <= *clock {
            break;
        }
        let next_request_instant = client.borrow_mut().check(limiter, *clock);
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
fn test_simple() {
    let now = Instant::now();
    let mut limiter =
        <FairRateLimiter<u8, 10>>::new(Duration::from_secs(1), 100, 25, Rand32::new(1), now)
            .unwrap();
    assert!(limiter.check(0, 99, now));
    assert!(limiter.check(1, 99, now));
    assert!(!limiter.check(0, 1, now));
    assert!(!limiter.check(0, 1, now));
}

#[test]
fn test_single_client() {
    let mut clock = Instant::now();
    let mut limiter = FairRateLimiter::new(
        Duration::from_secs(1),
        100,
        25,
        Rand32::new(1),
        Instant::now(),
    )
    .unwrap();
    for (seconds, rps, expected_accepted_requests) in [
        (100, 50, 5000..5001),
        (100, 75, 7501..7502),
        (100, 76, 7500..7700),
        (100, 77, 7600..7800),
        (100, 78, 7700..7900),
        (100, 79, 7800..8000),
        (100, 80, 7800..8100),
        (100, 81, 7900..8200),
        (100, 82, 8000..8300),
        (100, 83, 8100..8400),
        (100, 84, 8200..8500),
        (100, 85, 8300..8600),
        (100, 86, 8300..8600),
        (100, 87, 8400..8700),
        (100, 88, 8500..8800),
        (100, 89, 8600..8900),
        (100, 90, 8600..8900),
        (100, 91, 8700..9000),
        (100, 92, 8700..9000),
        (100, 93, 8800..9100),
        (100, 94, 8800..9100),
        (100, 95, 8900..9200),
        (100, 96, 8900..9200),
        (100, 97, 8900..9200),
        (100, 98, 9000..9300),
        (100, 99, 9000..9300),
        (100, 100, 9100..9400),
        (100, 150, 9800..10100),
        (100, 500, 9900..10200),
    ] {
        let client = Rc::new(RefCell::new(Client::new(Key::Static(0), rps, 1)));
        simulate(&mut limiter, &mut clock, seconds, &[&client]);
        assert_contains!(
            expected_accepted_requests,
            client.borrow().accepted_requests
        );
    }
}

#[test]
fn test_four_clients() {
    let mut clock = Instant::now();
    let mut limiter = FairRateLimiter::new(
        Duration::from_secs(1),
        100,
        25,
        Rand32::new(1),
        Instant::now(),
    )
    .unwrap();
    for ((rps0, rps1, rps2, rps3), (exp0, exp1, exp2, exp3), exp_sum) in [
        (
            (50, 25, 5, 1),
            (5000..=5000, 2500..=2500, 500..=500, 100..=100),
            8000..=8500,
        ),
        (
            (100, 50, 10, 1),
            (5000..=5500, 3500..=4000, 1000..=1000, 100..=100),
            9000..=10_000,
        ),
        (
            (200, 100, 20, 2),
            (4500..=5000, 3000..=4000, 2000..=2000, 200..=200),
            9000..=11_000,
        ),
        (
            (200, 200, 200, 17),
            (2500..=3500, 2500..=3500, 2500..=3500, 1700..=1701),
            9000..=11_000,
        ),
    ] {
        let client0 = Rc::new(RefCell::new(Client::new(Key::Static(0), rps0, 1)));
        let client1 = Rc::new(RefCell::new(Client::new(Key::Static(1), rps1, 1)));
        let client2 = Rc::new(RefCell::new(Client::new(Key::Static(2), rps2, 1)));
        let client3 = Rc::new(RefCell::new(Client::new(Key::Static(3), rps3, 1)));
        simulate(
            &mut limiter,
            &mut clock,
            100,
            &[&client0, &client1, &client2, &client3],
        );
        assert_contains!(exp0, client0.borrow().accepted_requests);
        assert_contains!(exp1, client1.borrow().accepted_requests);
        assert_contains!(exp2, client2.borrow().accepted_requests);
        assert_contains!(exp3, client3.borrow().accepted_requests);
        assert_contains!(
            exp_sum,
            client0.borrow().accepted_requests
                + client1.borrow().accepted_requests
                + client2.borrow().accepted_requests
                + client3.borrow().accepted_requests
        );
    }
}

#[test]
fn test_client_and_longtail() {
    let mut clock = Instant::now();
    let mut limiter = FairRateLimiter::new(
        Duration::from_secs(1),
        100,
        25,
        Rand32::new(1),
        Instant::now(),
    )
    .unwrap();
    for ((rps_client, rps_longtail), (exp_client, exp_longtail), exp_sum) in [
        (
            (25, 25),
            (25_000..=25_000, 25_000..=25_000),
            50_000..=50_000,
        ),
        (
            (75, 50),
            (49_000..=51_000, 49_000..=51_000),
            99_000..=101_000,
        ),
        (
            (100, 100),
            (29_000..=32_000, 95_000..=100_000),
            129_000..=131_000,
        ),
    ] {
        let client = Rc::new(RefCell::new(Client::new(Key::Static(0), rps_client, 1)));
        let longtail = Rc::new(RefCell::new(Client::new(
            Key::Random(Cell::new(Rand32::new(2))),
            rps_longtail,
            1,
        )));
        simulate(&mut limiter, &mut clock, 1000, &[&client, &longtail]);
        assert_contains!(exp_client, client.borrow().accepted_requests);
        assert_contains!(exp_longtail, longtail.borrow().accepted_requests);
        assert_contains!(
            exp_sum,
            client.borrow().accepted_requests + longtail.borrow().accepted_requests
        );
    }
}

use core::time::Duration;
use oorandom::Rand32;
use prob_rate_limiter::ProbRateLimiter;
use std::time::Instant;

#[derive(Debug)]
struct Client {
    rps: u32,
    cost: u32,
    pub accepted_requests: u32,
}
impl Client {
    pub fn new(rps: u32, cost: u32) -> Self {
        Self {
            rps,
            cost,
            accepted_requests: 0,
        }
    }

    fn check(&mut self, limiter: &mut ProbRateLimiter, now: Instant) -> Instant {
        if limiter.check(self.cost, now) {
            self.accepted_requests += 1;
        }
        now + (Duration::from_secs(1) / self.rps)
    }
}

fn simulate(
    limiter: &mut ProbRateLimiter,
    clock: &mut Instant,
    num_seconds: u64,
    client: &mut Client,
) {
    let deadline = *clock + Duration::from_secs(num_seconds);
    let mut next_request_instant = *clock;
    let mut num_requests = 0;
    loop {
        assert!(*clock <= next_request_instant);
        *clock = next_request_instant;
        if deadline <= *clock {
            break;
        }
        next_request_instant = client.check(limiter, *clock);
        num_requests += 1;
    }
    println!(
        "Simulated {} request over {} seconds",
        num_requests, num_seconds
    );
    println!("client: {:?}", client);
}

macro_rules! assert_contains {
    ( $range:expr, $value:expr ) => {
        if !$range.contains(&$value) {
            panic!("{:?} is not in {:?}", $value, $range);
        }
    };
}

#[test]
fn test_new_custom() {
    let now = Instant::now();
    ProbRateLimiter::new_custom(Duration::from_nanos(1), 100, now, Rand32::new(1)).unwrap_err();
    let mut limiter =
        ProbRateLimiter::new_custom(Duration::from_secs(1), 100, now, Rand32::new(1)).unwrap();
    assert!(limiter.check(100, now));
    assert!(limiter.check(100, now));
    assert!(!limiter.check(1, now));
    assert!(!limiter.check(1, now));
}

#[test]
fn test_new() {
    let now = Instant::now();
    ProbRateLimiter::new(f32::NEG_INFINITY).unwrap_err();
    ProbRateLimiter::new(-1.0).unwrap_err();
    ProbRateLimiter::new(-0.0).unwrap_err();
    ProbRateLimiter::new(0.1).unwrap_err();
    ProbRateLimiter::new(f32::INFINITY).unwrap_err();
    ProbRateLimiter::new(f32::NAN).unwrap_err();
    let mut limiter = ProbRateLimiter::new(100.0).unwrap();
    assert!(limiter.check(100, now));
    assert!(limiter.check(100, now));
    assert!(!limiter.check(1, now));
    assert!(!limiter.check(1, now));
}

#[test]
fn test_zero() {
    let now = Instant::now();
    let mut limiter = ProbRateLimiter::new(0.0).unwrap();
    assert!(!limiter.check(1, now));
    assert!(!limiter.check(1, now));
}

#[test]
fn test_debug() {
    let limiter =
        ProbRateLimiter::new_custom(Duration::from_secs(7), 11, Instant::now(), Rand32::new(1))
            .unwrap();
    let debug_string = format!("{:?}", limiter);
    assert!(debug_string.contains("ProbRateLimiter"), "{}", debug_string);
    assert!(debug_string.contains("22"), "{}", debug_string);
    assert!(debug_string.contains("7"), "{}", debug_string);
}

#[test]
fn test_clone() {
    let mut limiter = ProbRateLimiter::new(1.0).unwrap();
    let mut clone = limiter.clone();
    assert_eq!(format!("{:?}", limiter), format!("{:?}", clone));
    let now = Instant::now();
    assert!(limiter.check(1, now));
    assert!(limiter.check(1, now));
    assert!(!limiter.check(1, now));
    assert!(clone.check(1, now));
    assert!(clone.check(1, now));
    assert!(!clone.check(1, now));
}

// TODO: Test `attempt` and `record` separately.

#[test]
fn test_steady_state() {
    let mut clock = Instant::now();
    let mut limiter =
        ProbRateLimiter::new_custom(Duration::from_secs(1), 100, clock, Rand32::new(1)).unwrap();
    for (rps, expected_accepted_requests) in [
        (50, 5000..5001),
        (75, 7501..7502),
        (76, 7500..7700),
        (77, 7600..7800),
        (78, 7700..7900),
        (79, 7800..8000),
        (80, 7800..8100),
        (81, 7900..8200),
        (82, 8000..8300),
        (83, 8100..8400),
        (84, 8200..8500),
        (85, 8300..8600),
        (86, 8300..8600),
        (87, 8400..8700),
        (88, 8500..8800),
        (89, 8600..8900),
        (90, 8600..8900),
        (91, 8700..9000),
        (92, 8700..9000),
        (93, 8800..9100),
        (94, 8800..9100),
        (95, 8900..9200),
        (96, 8900..9200),
        (97, 8900..9200),
        (98, 9000..9300),
        (99, 9000..9300),
        (100, 9100..9400),
        (150, 9800..10100),
        (500, 9900..10200),
    ] {
        let mut client = Client::new(rps, 1);
        simulate(&mut limiter, &mut clock, 100, &mut client);
        assert_contains!(expected_accepted_requests, client.accepted_requests);
    }
}

#[test]
fn test_bursty() {
    let mut clock = Instant::now();
    let mut limiter =
        ProbRateLimiter::new_custom(Duration::from_secs(1), 100, clock, Rand32::new(1)).unwrap();
    for (seconds, rps, expected_accepted_requests) in [
        (1, 50, 50..51),
        (1, 500, 99..190),
        (1, 50, 50..51),
        (1, 500, 99..190),
        (1, 50, 50..51),
        (1, 500, 99..190),
        (1, 50, 50..51),
        (1, 500, 99..190),
        (10, 50, 500..501),
        (10, 500, 990..1100),
        (10, 50, 500..501),
        (10, 500, 990..1100),
        (10, 50, 500..501),
        (10, 500, 990..1100),
        (100, 50, 5000..5001),
        (100, 500, 9900..10200),
        (100, 50, 5000..5001),
        (100, 500, 9900..10200),
        (100, 50, 5000..5001),
        (100, 500, 9900..10200),
    ] {
        let mut client = Client::new(rps, 1);
        simulate(&mut limiter, &mut clock, seconds, &mut client);
        assert_contains!(expected_accepted_requests, client.accepted_requests);
    }
}

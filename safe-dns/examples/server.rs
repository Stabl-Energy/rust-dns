// $ cargo run --package safe-dns --example server

use permit::Permit;
use prob_rate_limiter::ProbRateLimiter;
use safe_dns::DnsRecord;
use std::io::Read;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket};

fn main() {
    let permit = Permit::new();
    let serve_udp_permit = permit.new_sub();
    let sock = UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
    let addr = sock.local_addr().unwrap();
    println!("Bound UDP socket {}", addr.port());
    let response_bytes_rate_limiter = ProbRateLimiter::new(100_000);
    let records = vec![
        DnsRecord::new_a("aaa.example.com", "10.0.0.1").unwrap(),
        DnsRecord::new_aaaa("aaa.example.com", "2606:2800:220:1:248:1893:25c8:1946").unwrap(),
        DnsRecord::new_cname("bbb.example.com", "ccc.example.com").unwrap(),
    ];
    let join_handle = std::thread::spawn(move || {
        safe_dns::serve_udp(
            &serve_udp_permit,
            &sock,
            response_bytes_rate_limiter,
            &records,
        )
        .unwrap();
    });
    while std::io::stdin().read(&mut [0u8]).is_ok() {}
    permit.revoke();
    join_handle.join().unwrap();
}

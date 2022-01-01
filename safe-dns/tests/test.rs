use permit::Permit;
use safe_dns::DnsRecord;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket};
use std::process::Command;
use std::time::Duration;

#[test]
fn example() {
    let permit = Permit::new();
    let serve_udp_permit = permit.new_sub();
    let sock = UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
    let addr = sock.local_addr().unwrap();
    let records = vec![
        DnsRecord::new_a("aaa.example.com", "10.0.0.1").unwrap(),
        DnsRecord::new_aaaa("bbb.example.com", "2606:2800:220:1:248:1893:25c8:1946").unwrap(),
        DnsRecord::new_cname("ccc.example.com", "ddd.example.com").unwrap(),
    ];
    let join_handle = std::thread::spawn(move || {
        safe_dns::serve_udp(&serve_udp_permit, &sock, &records).unwrap()
    });
    assert!(Command::new("dig")
        .arg("@localhost")
        .arg("-p")
        .arg(addr.port().to_string())
        .arg("+time=1")
        .arg("a")
        .arg("aaa.example.com")
        .status()
        .unwrap()
        .success());
    assert!(Command::new("dig")
        .arg("@localhost")
        .arg("-p")
        .arg(addr.port().to_string())
        .arg("+time=1")
        .arg("aaaa")
        .arg("bbb.example.com")
        .status()
        .unwrap()
        .success());
    assert!(Command::new("dig")
        .arg("@localhost")
        .arg("-p")
        .arg(addr.port().to_string())
        .arg("+time=1")
        .arg("cname")
        .arg("ccc.example.com")
        .status()
        .unwrap()
        .success());
    permit.revoke();
    join_handle.join().unwrap();
}

#[test]
fn hard_coded() {
    let permit = Permit::new();
    let serve_udp_permit = permit.new_sub();
    let server_sock =
        UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
    let addr = server_sock.local_addr().unwrap();
    let records = vec![DnsRecord::new_a("aaa.example.com", "10.0.0.1").unwrap()];
    let join_handle = std::thread::spawn(move || {
        safe_dns::serve_udp(&serve_udp_permit, &server_sock, &records).unwrap()
    });
    let request = vec![
        0x9A, 0x9A, 1, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 97, 97, 97,
        0x07, 101, 120, 97, 109, 112, 108, 101, 0x03, 99, 111, 109, 0x00, 0x00, 0x01, 0x00, 0x01,
    ];
    let expected_response = vec![
        0x9A, 0x9A, 0x85, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 97, 97, 97,
        0x07, 101, 120, 97, 109, 112, 108, 101, 0x03, 99, 111, 109, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x03, 97, 97, 97, 0x07, 101, 120, 97, 109, 112, 108, 101, 0x03, 99, 111, 109, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0x2C, 0x00, 0x04, 10, 0, 0, 1,
    ];
    let client_sock =
        UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
    client_sock
        .set_write_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    client_sock
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    client_sock.connect(&addr).unwrap();
    client_sock.send(&request).unwrap();
    let mut buf = [0_u8; 512];
    let response_len = client_sock.recv(&mut buf).unwrap();
    let response = &buf[0..response_len];
    assert_eq!(expected_response, response);
    permit.revoke();
    join_handle.join().unwrap();
}

// https://github.com/m-ou-se/single-use-dns
// https://crates.io/crates/dns-parser/0.8.0
// https://docs.rs/rusty_dns/0.0.3/rusty_dns/index.html

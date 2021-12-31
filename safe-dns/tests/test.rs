use permit::Permit;
use safe_dns::DnsRecord;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

#[test]
fn test() {
    let permit = Permit::new();
    let serve_udp_permit = permit.new_sub();
    let sock = UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
    let records = vec![
        DnsRecord::new_a("hello.example.com", "10.0.0.1").unwrap(),
        DnsRecord::new_aaaa("hello.example.com", "2606:2800:220:1:248:1893:25c8:1946").unwrap(),
        DnsRecord::new_cname("abc.example.com", "def.example.com").unwrap(),
    ];
    let join_handle = std::thread::spawn(move || {
        safe_dns::serve_udp(&serve_udp_permit, &sock, &records).unwrap()
    });
    permit.revoke();
    join_handle.join().unwrap();
}

#[test]
fn test_binary() {
    let permit = Permit::new();
    let serve_udp_permit = permit.new_sub();
    let server_sock =
        UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
    let addr = server_sock.local_addr().unwrap();
    let records = vec![
        DnsRecord::new_a("hello.example.com", "10.0.0.1").unwrap(),
        DnsRecord::new_aaaa("hello.example.com", "2606:2800:220:1:248:1893:25c8:1946").unwrap(),
        DnsRecord::new_cname("abc.example.com", "def.example.com").unwrap(),
    ];
    let join_handle = std::thread::spawn(move || {
        safe_dns::serve_udp(&serve_udp_permit, &server_sock, &records).unwrap()
    });
    // https://courses.cs.duke.edu//fall16/compsci356/DNS/DNS-primer.pdf
    let request = vec![
        0xdb, 0x42, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77, 0x77,
        0x77, 0x0c, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74, 0x65, 0x72, 0x6e, 0x03,
        0x65, 0x64, 0x75, 0x00, 0x00, 0x01, 0x00, 0x01,
    ];
    let expected_response = vec![
        0xdb, 0x42, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77, 0x77,
        0x77, 0x0c, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74, 0x65, 0x72, 0x6e, 0x03,
        0x65, 0x64, 0x75, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00,
        0x00, 0x02, 0x58, 0x00, 0x04, 0x9b, 0x21, 0x11, 0x44,
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

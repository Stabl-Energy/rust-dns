use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[test]
fn test() {
    // let dns_server = DnsServer::new(
    //     SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 5053),
    //     [
    //         DnsRecord::A("hello.example.com", "10.0.0.1".into()),
    //         DnsRecord::AAAA(
    //             "hello.example.com",
    //             "2606:2800:220:1:248:1893:25c8:1946".into(),
    //         ),
    //         DnsRecord::CNAME("abc.example.com", "def.example.com"),
    //         // DnsRecord::SOA("abc.example.com", "def.example.com"),
    //     ],
    // )
    // .unwrap();
}

// https://github.com/m-ou-se/single-use-dns
// https://crates.io/crates/dns-parser/0.8.0
// https://docs.rs/rusty_dns/0.0.3/rusty_dns/index.html

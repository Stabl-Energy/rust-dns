dns-server
========
[![crates.io version](https://img.shields.io/crates/v/dns-server.svg)](https://crates.io/crates/dns-server)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/dns-server/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

A threaded DNS server library.

# Use Cases
- Make your API server its own DNS server.
  This eliminates the DNS server as a separate point of failure.
- Keep your DNS config in code, next to your server code.
  Include it in code reviews and integration tests.
- DNS-based
  [domain validation for free ACME certificates](https://letsencrypt.org/how-it-works/).
  This is useful for servers that don't listen on port 80.
  Servers on port 80 can use HTTP for domain validation and don't need to use this.

# Features
- Depends only on `std`
- `forbid(unsafe_code)`
- ?% test coverage

# Limitations
- Brand new.

# Example
```rust
use permit::Permit;
use prob_rate_limiter::ProbRateLimiter;
use dns_server::DnsRecord;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket};

let permit = Permit::new();
let sock = UdpSocket::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)).unwrap();
let addr = sock.local_addr().unwrap();
let response_bytes_rate_limiter = ProbRateLimiter::new(100_000);
let records = vec![
    DnsRecord::new_a("aaa.example.com", "93.184.216.34").unwrap(),
    DnsRecord::new_aaaa("aaa.example.com", "2606:2800:220:1:248:1893:25c8:1946").unwrap(),
    DnsRecord::new_cname("bbb.example.com", "target.foo.com").unwrap(),
];
dns_server::serve_udp(
    &permit,
    &sock,
    response_bytes_rate_limiter,
    &records,
)
.unwrap();
```

# Related Crates

# Cargo Geiger Safety Report
```

Metric output format: x/y
    x = unsafe code used by the build
    y = total unsafe code found in the crate

Symbols: 
    🔒  = No `unsafe` usage found, declares #![forbid(unsafe_code)]
    ❓  = No `unsafe` usage found, missing #![forbid(unsafe_code)]
    ☢️  = `unsafe` usage found

Functions  Expressions  Impls  Traits  Methods  Dependency

0/0        0/0          0/0    0/0     0/0      🔒  dns-server 0.1.0
0/0        0/0          0/0    0/0     0/0      🔒  ├── fixed-buffer 0.3.1
0/0        0/0          0/0    0/0     0/0      🔒  ├── multimap 0.8.3
0/0        5/5          0/0    0/0     0/0      ☢️  │   └── serde 1.0.136
0/0        0/0          0/0    0/0     0/0      ❓  │       └── serde_derive 1.0.136
0/0        0/12         0/0    0/0     0/3      ❓  │           ├── proc-macro2 1.0.36
0/0        0/0          0/0    0/0     0/0      🔒  │           │   └── unicode-xid 0.2.2
0/0        0/0          0/0    0/0     0/0      ❓  │           ├── quote 1.0.16
0/0        0/12         0/0    0/0     0/3      ❓  │           │   └── proc-macro2 1.0.36
0/0        0/47         0/3    0/0     0/2      ❓  │           └── syn 1.0.89
0/0        0/12         0/0    0/0     0/3      ❓  │               ├── proc-macro2 1.0.36
0/0        0/0          0/0    0/0     0/0      ❓  │               ├── quote 1.0.16
0/0        0/0          0/0    0/0     0/0      🔒  │               └── unicode-xid 0.2.2
0/0        0/0          0/0    0/0     0/0      🔒  ├── permit 0.1.4
0/0        0/0          0/0    0/0     0/0      🔒  └── prob-rate-limiter 0.1.1
0/0        0/0          0/0    0/0     0/0      🔒      └── oorandom 11.1.3

0/0        5/64         0/3    0/0     0/5    

```
# Changelog
- v0.1.0 - Initial version

# To Do
- Message compression
- Decide whether to send back error responses.
- Ergonomic constructors that take `OsStr`, for using environment variables
- Custom TTLs
- NS records (and glue)
- Client
- Caching client
- Recursive resolver

# Alternatives


License: Apache-2.0

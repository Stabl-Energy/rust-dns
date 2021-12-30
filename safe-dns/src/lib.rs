//! [![crates.io version](https://img.shields.io/crates/v/safe-dns.svg)](https://crates.io/crates/safe-dns)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/safe-dns/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # safe-dns
//!
//! A threaded DNS server library.
//!
//! ## Use Cases
//! - Make your API server its own DNS server.
//!   This eliminates the DNS server as a separate point of failure.
//! - Keep your DNS config in code, next to your server code.
//!   Include it in code reviews and integration tests.
//! - DNS-based
//!   [domain validation for free ACME certificates](https://letsencrypt.org/how-it-works/).
//!   This is useful for servers that don't listen on port 80.
//!   Servers on port 80 can use HTTP for domain validation and don't need to use this.
//!
//! ## Features
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//! - ?% test coverage
//!
//! ## Limitations
//!
//! ## Example
//!
//! ## Related Crates
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.0 - Initial version
//!
//! # To Do
//! - Ergonomic constructors that take `OsStr`, for using environment variables
//!
//! ## Alternatives
//!
#![forbid(unsafe_code)]

use core::fmt::Display;
use fixed_buffer::FixedBuf;
use std::fmt::Formatter;
use std::io::ErrorKind;
use std::net::IpAddr;
use std::time::Duration;

/// A name that conforms to the conventions in
/// [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035#section-2.3.1):
///
/// > 2.3.1. Preferred name syntax
/// >
/// > The DNS specifications attempt to be as general as possible in the rules for constructing
/// > domain names.  The idea is that the name of any existing object can be expressed as a domain
/// > name with minimal changes.
/// >
/// > However, when assigning a domain name for an object, the prudent user will select a name
/// > which satisfies both the rules of the domain system and any existing rules for the object,
/// > whether these rules are published or implied by existing programs.
/// >
/// > For example, when naming a mail domain, the user should satisfy both the rules of this memo
/// > and those in [RFC-822](https://datatracker.ietf.org/doc/html/rfc822).  When creating a new
/// > host name, the old rules for HOSTS.TXT should be followed.  This avoids problems when old
/// > software is converted to use domain names.
/// >
/// > The following syntax will result in fewer problems with many
/// >
/// > applications that use domain names (e.g., mail, TELNET).
/// >
/// > `<domain> ::= <subdomain> | " "`
/// >
/// > `<subdomain> ::= <label> | <subdomain> "." <label>`
/// >
/// > `<label> ::= <letter> [ [ <ldh-str> ] <let-dig> ]`
/// >
/// > `<ldh-str> ::= <let-dig-hyp> | <let-dig-hyp> <ldh-str>`
/// >
/// > `<let-dig-hyp> ::= <let-dig> | "-"`
/// >
/// > `<let-dig> ::= <letter> | <digit>`
/// >
/// > `<letter> ::=` any one of the 52 alphabetic characters `A` through `Z` in upper case
/// > and `a` through `z` in lower case
/// >
/// > `<digit> ::=` any one of the ten digits `0` through `9`
/// >
/// > Note that while upper and lower case letters are allowed in domain names, no significance is
/// > attached to the case.  That is, two names with the same spelling but different case are to be
/// > treated as if identical.
/// >
/// > The labels must follow the rules for ARPANET host names.  They must start with a letter, end
/// > with a letter or digit, and have as interior characters only letters, digits, and hyphen.
/// > There are also some restrictions on the length.  Labels must be 63 characters or less.
/// >
/// > For example, the following strings identify hosts in the Internet:
/// >
/// > `A.ISI.EDU XX.LCS.MIT.EDU SRI-NIC.ARPA`
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DnsName(String);
impl DnsName {
    fn err(value: &str) -> Result<Self, String> {
        Err(format!("not a valid DNS name: {:?}", value))
    }

    fn is_letter(b: u8) -> bool {
        (b'a'..=b'z').contains(&b) || (b'A'..=b'Z').contains(&b)
    }

    fn is_letter_digit(b: u8) -> bool {
        Self::is_letter(b) || (b'0'..=b'9').contains(&b)
    }

    fn is_letter_digit_hyphen(b: u8) -> bool {
        Self::is_letter_digit(b) || b == b'-'
    }

    fn is_valid_label(label: &str) -> bool {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        let bytes = label.as_bytes();
        Self::is_letter(bytes[0])
            && bytes.iter().copied().all(Self::is_letter_digit_hyphen)
            && Self::is_letter_digit(*bytes.last().unwrap())
    }

    fn is_valid_name(value: &str) -> bool {
        if !value.is_ascii() {
            return false;
        }
        value.split('.').all(Self::is_valid_label)
    }

    /// # Errors
    /// Returns an error when `value` is not a valid DNS name.
    pub fn new(value: &str) -> Result<Self, String> {
        let trimmed = value.strip_suffix('.').unwrap_or(value);
        if trimmed.len() > 255 || !Self::is_valid_name(trimmed) {
            return Self::err(value);
        }
        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    #[must_use]
    pub fn inner(&self) -> &str {
        &self.0
    }
}
impl Display for DnsName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
impl std::convert::TryFrom<&'static str> for DnsName {
    type Error = String;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        DnsName::new(value)
    }
}
#[cfg(test)]
#[test]
fn test_dns_name() {
    // Err
    assert_eq!(
        <Result<DnsName, String>>::Err("not a valid DNS name: \"abc!\"".to_string()),
        DnsName::new("abc!")
    );
    // Separators.
    DnsName::new(".").unwrap_err();
    assert_eq!("a", DnsName::new("a.").unwrap().inner());
    DnsName::new("a..").unwrap_err();
    DnsName::new(".a").unwrap_err();
    DnsName::new("b..a").unwrap_err();
    DnsName::new(".b.a").unwrap_err();
    // Labels.
    assert_eq!(
        <Result<DnsName, String>>::Err("not a valid DNS name: \"a\u{263A}\"".to_string()),
        DnsName::new("a\u{263A}")
    );
    assert_eq!("a", DnsName::new("a").unwrap().inner());
    assert_eq!("b", DnsName::new("b").unwrap().inner());
    assert_eq!("z", DnsName::new("z").unwrap().inner());
    assert_eq!("abc", DnsName::new("ABC").unwrap().inner());
    assert_eq!("b", DnsName::new("B").unwrap().inner());
    assert_eq!("z", DnsName::new("Z").unwrap().inner());
    assert_eq!(
        "abcdefghijklmnopqrstuvwxyz",
        DnsName::new("abcdefghijklmnopqrstuvwxyz").unwrap().inner()
    );
    DnsName::new("1").unwrap_err();
    DnsName::new("1a").unwrap_err();
    assert_eq!("a0", DnsName::new("a0").unwrap().inner());
    assert_eq!("a1", DnsName::new("a1").unwrap().inner());
    assert_eq!("a9", DnsName::new("a9").unwrap().inner());
    assert_eq!("a9876543210", DnsName::new("a9876543210").unwrap().inner());
    DnsName::new("-").unwrap_err();
    DnsName::new("a-").unwrap_err();
    DnsName::new("-a").unwrap_err();
    DnsName::new("a-.b").unwrap_err();
    DnsName::new("a.-b").unwrap_err();
    assert_eq!("a-b", DnsName::new("a-b").unwrap().inner());
    assert_eq!("a-0", DnsName::new("a-0").unwrap().inner());
    assert_eq!("a---b", DnsName::new("a---b").unwrap().inner());
    assert_eq!(
        "xyz321-654abc",
        DnsName::new("Xyz321-654abC").unwrap().inner()
    );
    // Length
    DnsName::new("").unwrap_err();
    DnsName::new("a").unwrap();
    DnsName::new("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    DnsName::new("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap_err();
    DnsName::new(concat!(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    ))
    .unwrap();
    DnsName::new(concat!(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.a"
    ))
    .unwrap_err();
    // inner
    assert_eq!("abc", DnsName::new("abc").unwrap().inner());
    // Display
    assert_eq!(
        "example.com",
        format!("{}", DnsName::new("example.com").unwrap())
    );
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DnsRecord {
    A(DnsName, std::net::Ipv4Addr),
    AAAA(DnsName, std::net::Ipv6Addr),
    CNAME(DnsName, DnsName),
}
impl DnsRecord {
    /// # Errors
    /// Returns an error when `name` is not a valid DNS name
    /// or `ipv4_addr` is not a valid IPv4 address.
    pub fn new_a(name: &str, ipv4_addr: &str) -> Result<Self, String> {
        let dns_name = DnsName::new(name)?;
        let ip_addr: IpAddr = ipv4_addr
            .parse()
            .map_err(|e| format!("failed parsing {:?} as an IP address: {}", ipv4_addr, e))?;
        match ip_addr {
            IpAddr::V4(addr) => Ok(Self::A(dns_name, addr)),
            IpAddr::V6(addr) => Err(format!(
                "cannot create an A record with ipv6 address {:?}",
                addr
            )),
        }
    }

    /// # Errors
    /// Returns an error when `name` is not a valid DNS name
    /// or `ipv6_addr` is not a valid IPv6 address.
    pub fn new_aaaa(name: &str, ipv6_addr: &str) -> Result<Self, String> {
        let dns_name = DnsName::new(name)?;
        let ip_addr: IpAddr = ipv6_addr
            .parse()
            .map_err(|e| format!("failed parsing {:?} as an IP address: {}", ipv6_addr, e))?;
        match ip_addr {
            IpAddr::V4(addr) => Err(format!(
                "cannot create an AAAA record with ipv4 address {:?}",
                addr
            )),
            IpAddr::V6(addr) => Ok(Self::AAAA(dns_name, addr)),
        }
    }

    /// # Errors
    /// Returns an error when `name` or `target` are not both valid DNS names.
    pub fn new_cname(name: &str, target: &str) -> Result<Self, String> {
        let dns_name = DnsName::new(name)?;
        let dns_name_target = DnsName::new(target)?;
        Ok(Self::CNAME(dns_name, dns_name_target))
    }
}
impl core::fmt::Debug for DnsRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            DnsRecord::A(name, addr) => write!(f, "DnsRecord::A({},{})", name, addr),
            DnsRecord::AAAA(name, addr) => write!(f, "DnsRecord::AAAA({},{})", name, addr),
            DnsRecord::CNAME(name, target) => write!(f, "DnsRecord::CNAME({},{})", name, target),
        }
    }
}
#[cfg(test)]
#[test]
fn test_dns_record() {
    use std::net::{Ipv4Addr, Ipv6Addr};
    // Constructors
    assert_eq!(
        DnsRecord::A(DnsName::new("a.b").unwrap(), Ipv4Addr::new(1, 2, 3, 4)),
        DnsRecord::new_a("a.b", "1.2.3.4").unwrap()
    );
    assert_eq!(
        DnsRecord::AAAA(
            DnsName::new("a.b").unwrap(),
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)
        ),
        DnsRecord::new_aaaa("a.b", "2001:db8::").unwrap()
    );
    assert_eq!(
        DnsRecord::CNAME(DnsName::new("a.b").unwrap(), DnsName::new("c.d").unwrap()),
        DnsRecord::new_cname("a.b", "c.d").unwrap()
    );
    // Debug
    assert_eq!(
        "DnsRecord::A(a.b,1.2.3.4)",
        format!(
            "{:?}",
            DnsRecord::A(DnsName::new("a.b").unwrap(), Ipv4Addr::new(1, 2, 3, 4))
        )
    );
    assert_eq!(
        "DnsRecord::AAAA(a.b,2001:db8::)",
        format!(
            "{:?}",
            DnsRecord::AAAA(
                DnsName::new("a.b").unwrap(),
                Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)
            )
        )
    );
    assert_eq!(
        "DnsRecord::CNAME(a.b,c.d)",
        format!(
            "{:?}",
            DnsRecord::CNAME(DnsName::new("a.b").unwrap(), DnsName::new("c.d").unwrap())
        )
    );
}

#[derive(Debug, PartialEq)]
enum ProcessError {
    Truncated,
    NotFound,
}

fn process_datagram(
    _records: &[DnsRecord],
    _in_bytes: &[u8],
    _out_bytes: &mut FixedBuf<65507>,
) -> Result<(), String> {
    todo!()
}

pub fn serve_udp(
    sock: std::net::UdpSocket,
    records: &[DnsRecord],
    permit: permit::Permit,
) -> Result<(), String> {
    sock.set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|e| format!("error setting socket read timeout: {}", e))?;
    let local_addr = sock
        .local_addr()
        .map_err(|e| format!("error getting socket local address: {}", e))?;
    // Buffer is the maximum size of an IPv4 UDP payload.  This does not support IPv6 jumbograms.
    let mut read_buf = [0u8; 65507];
    let mut write_buf: FixedBuf<65507> = FixedBuf::new();
    while !permit.is_revoked() {
        let (in_bytes, addr) = match sock.recv_from(&mut read_buf) {
            Ok((len, _)) if len > read_buf.len() => continue, // Discard jumbogram.
            Ok((len, addr)) => (&read_buf[0..len], addr),
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                continue
            }
            Err(e) => return Err(format!("error reading socket {:?}: {}", local_addr, e)),
        };
        write_buf.clear();
        if process_datagram(records, in_bytes, &mut write_buf).is_err() {
            continue;
        }
        if write_buf.is_empty() {
            unreachable!();
        }
        let sent_len = sock
            .send_to(write_buf.readable(), &addr)
            .map_err(|e| format!("error sending response to {:?}: {}", addr, e))?;
        if sent_len != write_buf.len() {
            return Err(format!(
                "sent only {} bytes of {} byte response to {:?}",
                sent_len,
                write_buf.len(),
                addr
            ));
        }
    }
    Ok(())
}

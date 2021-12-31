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
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::io::ErrorKind;
use std::iter::FromIterator;
use std::net::IpAddr;
use std::time::Duration;

const INTERNET_CLASS: u16 = 1;
const ANY_CLASS: u16 = 255;

fn read_exact<const N: usize, const M: usize>(
    buf: &mut FixedBuf<N>,
) -> Result<[u8; M], ProcessError> {
    let mut result = [0_u8; M];
    buf.try_read_exact(&mut result)
        .ok_or(ProcessError::Truncated)?;
    Ok(result)
}

fn read_byte<const N: usize>(buf: &mut FixedBuf<N>) -> Result<u8, ProcessError> {
    buf.try_read_byte().ok_or(ProcessError::Truncated)
}

fn write_u16_be<const N: usize>(out: &mut FixedBuf<N>, value: u16) -> Result<(), ProcessError> {
    let bytes: [u8; 2] = value.to_be_bytes();
    out.write_bytes(&bytes)
        .map_err(|_| ProcessError::ResponseBufferFull)?;
    Ok(())
}

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
            return Err(format!("not a valid DNS name: {:?}", value));
        }
        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    pub fn read<const N: usize>(buf: &mut FixedBuf<N>) -> Result<DnsName, ProcessError> {
        let mut value = String::new();
        for _ in 0..63 {
            let len = read_byte(buf)? as usize;
            if len == 0 {
                if value.is_empty() {
                    return Err(ProcessError::EmptyName);
                }
                if value.len() > 255 {
                    return Err(ProcessError::NameTooLong);
                }
                return Ok(Self(value));
            }
            if buf.readable().len() < len {
                return Err(ProcessError::Truncated);
            }
            let label_bytes = &buf.readable()[0..len];
            let label = std::str::from_utf8(label_bytes).map_err(|_| ProcessError::InvalidLabel)?;
            if !Self::is_valid_label(label) {
                return Err(ProcessError::InvalidLabel);
            }
            if !value.is_empty() {
                value.push('.');
            }
            value.push_str(label);
        }
        Err(ProcessError::TooManyLabels)
    }

    pub fn write<const N: usize>(&self, out: &mut FixedBuf<N>) -> Result<(), ProcessError> {
        for label in self.0.split('.') {
            if label.len() > 63 {
                unreachable!();
            }
            let len = u8::try_from(label.len()).unwrap();
            out.write_bytes(&[len])
                .map_err(|_| ProcessError::ResponseBufferFull)?;
            out.write_bytes(label.as_bytes())
                .map_err(|_| ProcessError::ResponseBufferFull)?;
        }
        out.write_bytes(&[0])
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        Ok(())
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

    pub fn name(&self) -> &DnsName {
        match self {
            DnsRecord::A(dns_name, _)
            | DnsRecord::AAAA(dns_name, _)
            | DnsRecord::CNAME(dns_name, _) => dns_name,
        }
    }

    pub fn typ(&self) -> Type {
        match self {
            DnsRecord::A(_, _) => Type::A,
            DnsRecord::AAAA(_, _) => Type::AAAA,
            DnsRecord::CNAME(_, _) => Type::CNAME,
        }
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
pub enum ProcessError {
    EmptyName,
    InvalidClass,
    InvalidLabel,
    InvalidOpCode,
    NameTooLong,
    NoQuestion,
    NotARequest,
    NotFound,
    ResponseBufferFull,
    QueryHasAdditionalRecords,
    QueryHasAnswer,
    QueryHasNameServer,
    TooManyAdditional,
    TooManyAnswers,
    TooManyLabels,
    TooManyNameServers,
    TooManyQuestions,
    Truncated,
}

/// > TYPE fields are used in resource records.  Note that these types are a subset of QTYPEs.
///
/// <https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2>
///
/// > A record type is defined to store a host's IPv6 address.  A host that has more than one
/// > IPv6 address must have more than one such record.
///
/// <https://datatracker.ietf.org/doc/html/rfc3596#section-2>
///
/// > QTYPE fields appear in the question part of a query.  QTYPES are a superset of TYPEs, hence
/// > all TYPEs are valid QTYPEs.
///
/// <https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.3>
#[derive(Debug, PartialEq)]
pub enum Type {
    /// IPv4 address
    A,
    /// IPv6 address
    AAAA,
    /// The canonical name for an alias
    CNAME,
    /// Mail exchange
    MX,
    /// Authoritative name server
    NS,
    /// Domain name pointer
    PTR,
    /// Marks the start of a zone of authority
    SOA,
    /// Text string
    TXT,
    Unknown(u16),
}
impl Type {
    pub fn new(value: u16) -> Self {
        match value {
            1 => Type::A,
            28 => Type::AAAA,
            5 => Type::CNAME,
            15 => Type::MX,
            2 => Type::NS,
            12 => Type::PTR,
            6 => Type::SOA,
            16 => Type::TXT,
            other => Type::Unknown(other),
        }
    }
    pub fn num(&self) -> u16 {
        match self {
            Type::A => 1,
            Type::AAAA => 28,
            Type::CNAME => 5,
            Type::MX => 15,
            Type::NS => 2,
            Type::PTR => 12,
            Type::SOA => 6,
            Type::TXT => 16,
            Type::Unknown(other) => *other,
        }
    }
}

/// > `OPCODE`  A four bit field that specifies kind of query in this message.
/// >         This value is set by the originator of a query and copied into
/// >         the response.  The values are:
/// > - `0` a standard query (`QUERY`)
/// > - `1` an inverse query (`IQUERY`)
/// > - `2` a server status request (`STATUS`)
/// > - `3-15` reserved for future use
///
/// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
#[derive(Debug, PartialEq)]
enum OpCode {
    Query,
    InverseQuery,
    Status,
    Reserved(u8),
}
impl OpCode {
    pub fn new(value: u8) -> Self {
        match value {
            0 => OpCode::Query,
            1 => OpCode::InverseQuery,
            2 => OpCode::Status,
            other => OpCode::Reserved(other),
        }
    }
    pub fn num(&self) -> u8 {
        match self {
            OpCode::Query => 0,
            OpCode::InverseQuery => 1,
            OpCode::Status => 2,
            OpCode::Reserved(other) => *other,
        }
    }
}

/// > `RCODE` Response code - this 4 bit field is set as part of responses.  The values have the
/// > following interpretation:
/// > - `0` No error condition
/// > - `1` Format error - The name server was unable to interpret the query.
/// > - `2` Server failure - The name server was unable to process this query due to a problem with
/// >   the name server.
/// > - `3` Name Error - Meaningful only for responses from an authoritative name server, this code
/// >   signifies that the domain name referenced in the query does not exist.
/// > - `4` Not Implemented - The name server does not support the requested kind of query.
/// > - `5` Refused - The name server refuses to perform the specified operation for policy reasons.
/// >   For example, a name server may not wish to provide the information to the particular
/// >   requester, or a name server may not wish to perform a particular operation (e.g., zone
/// >    transfer) for particular data.
/// > - `6-15` Reserved for future use.
///
/// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
#[derive(Debug, PartialEq)]
enum ResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserved(u8),
}
impl ResponseCode {
    pub fn new(value: u8) -> Self {
        match value {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            other => ResponseCode::Reserved(other),
        }
    }
    pub fn num(&self) -> u8 {
        match self {
            ResponseCode::NoError => 0,
            ResponseCode::FormatError => 1,
            ResponseCode::ServerFailure => 2,
            ResponseCode::NameError => 3,
            ResponseCode::NotImplemented => 4,
            ResponseCode::Refused => 5,
            ResponseCode::Reserved(other) => *other,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Question {
    name: DnsName,
    typ: Type,
}

struct Message {
    /// > `ID` A 16 bit identifier assigned by the program that generates any kind of query.  This
    /// > identifier is copied the corresponding reply and can be used by the requester to match up
    /// > replies to outstanding queries.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    id: u16,
    /// > `QR` A one bit field that specifies whether this message is a query (`0`),
    /// > or a response (`1`).
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    is_response: bool,
    op_code: OpCode,
    /// > `AA` Authoritative Answer - this bit is valid in responses, and specifies that the
    /// > responding name server is an authority for the domain name in question section.
    /// >
    /// > Note that the contents of the answer section may have multiple owner names because of
    /// > aliases.  The AA bit corresponds to the name which matches the query name, or the first
    /// > owner name in the answer section.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    authoritative_answer: bool,
    /// > `TC` TrunCation - specifies that this message was truncated due to length greater than
    /// > that permitted on the transmission channel.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    truncated: bool,
    /// > `RD` Recursion Desired - this bit may be set in a query and is copied into the response.
    /// > If RD is set, it directs the name server to pursue the query recursively.  Recursive query
    /// > support is optional.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    recursion_desired: bool,
    /// > `RA` Recursion Available - this be is set or cleared in a response, and denotes whether
    /// > recursive query support is available in the name server.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    recursion_available: bool,
    response_code: ResponseCode,
    questions: Vec<Question>,
    answers: Vec<DnsRecord>,
    name_servers: Vec<DnsRecord>,
    additional: Vec<DnsRecord>,
}
impl Message {
    pub fn parse<const N: usize>(mut buf: FixedBuf<N>) -> Result<Self, ProcessError> {
        // Header
        let bytes: [u8; 12] = read_exact(&mut buf)?;
        let id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let is_response = (bytes[2] >> 7) == 1;
        let op_code = OpCode::new((bytes[2] >> 3) & 0xF);
        let authoritative_answer = ((bytes[2] >> 2) & 1) == 1;
        let truncated = ((bytes[2] >> 1) & 1) == 1;
        let recursion_desired = (bytes[2] & 1) == 1;
        let recursion_available = (bytes[3] >> 7) == 1;
        let response_code = ResponseCode::new(bytes[3] & 0xF);
        let question_count = u16::from_be_bytes([bytes[4], bytes[5]]);
        let answer_count = u16::from_be_bytes([bytes[6], bytes[7]]);
        let name_server_count = u16::from_be_bytes([bytes[8], bytes[9]]);
        let additional_count = u16::from_be_bytes([bytes[10], bytes[11]]);
        if answer_count != 0 {
            return Err(ProcessError::QueryHasAnswer);
        }
        if name_server_count != 0 {
            return Err(ProcessError::QueryHasNameServer);
        }
        if additional_count != 0 {
            return Err(ProcessError::QueryHasAdditionalRecords);
        }
        // Questions
        let mut questions = Vec::with_capacity(question_count as usize);
        for _ in 0..question_count {
            let name = DnsName::read(&mut buf)?;
            let bytes: [u8; 4] = read_exact(&mut buf)?;
            let typ = Type::new(u16::from_be_bytes([bytes[0], bytes[1]]));
            let class = u16::from_be_bytes([bytes[2], bytes[3]]);
            if class != INTERNET_CLASS && class != ANY_CLASS {
                return Err(ProcessError::InvalidClass);
            }
            questions.push(Question { name, typ });
        }
        Ok(Self {
            id,
            is_response,
            op_code,
            authoritative_answer,
            truncated,
            recursion_desired,
            recursion_available,
            response_code,
            questions,
            answers: Vec::new(),
            name_servers: Vec::new(),
            additional: Vec::new(),
        })
    }

    pub fn write<const N: usize>(&self, out: &mut FixedBuf<N>) -> Result<(), ProcessError> {
        let bytes: [u8; 2] = self.id.to_be_bytes();
        out.write_bytes(&bytes)
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        let b = ((self.is_response as u8) << 7)
            & (self.op_code.num() << 3)
            & ((self.authoritative_answer as u8) << 2)
            & ((self.truncated as u8) << 1)
            & (self.recursion_desired as u8);
        out.write_bytes(&[b])
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        let b = ((self.recursion_available as u8) << 7) & self.response_code.num();
        out.write_bytes(&[b])
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        for count in [
            u16::try_from(self.questions.len()).map_err(|_| ProcessError::TooManyQuestions)?,
            u16::try_from(self.answers.len()).map_err(|_| ProcessError::TooManyAnswers)?,
            u16::try_from(self.name_servers.len()).map_err(|_| ProcessError::TooManyNameServers)?,
            u16::try_from(self.additional.len()).map_err(|_| ProcessError::TooManyAdditional)?,
        ] {
            write_u16_be(out, count)?;
        }
        if !self.questions.is_empty() {
            unimplemented!();
        }
        for record in self
            .answers
            .iter()
            .chain(self.name_servers.iter())
            .chain(self.additional.iter())
        {
            record.name().write(out)?;
            write_u16_be(out, record.typ().num())?;
            write_u16_be(out, INTERNET_CLASS)?;
            write_u16_be(out, 300_u16)?;
            // write_u16_be(out, rdlen)?;
            // write rdata
            todo!();
        }
        Ok(())
    }
}

fn process_datagram(
    name_to_record: &HashMap<&DnsName, &DnsRecord>,
    bytes: FixedBuf<512>,
    out: &mut FixedBuf<512>,
) -> Result<(), ProcessError> {
    let request = Message::parse(bytes)?;
    if request.is_response {
        return Err(ProcessError::NotARequest);
    }
    if request.op_code != OpCode::Query {
        return Err(ProcessError::InvalidOpCode);
    }
    // NOTE: We only answer the first question.
    let question = request.questions.first().ok_or(ProcessError::NoQuestion)?;
    let record = *name_to_record
        .get(&question.name)
        .ok_or(ProcessError::NotFound)?;
    if record.typ() != question.typ {
        return Err(ProcessError::NotFound);
    }
    let response = Message {
        id: request.id,
        is_response: true,
        op_code: request.op_code,
        authoritative_answer: true,
        truncated: false,
        recursion_desired: request.recursion_desired,
        recursion_available: false,
        response_code: ResponseCode::NoError,
        questions: Vec::new(),
        answers: vec![record.clone()],
        name_servers: Vec::new(),
        additional: Vec::new(),
    };
    response.write(out)?;
    Ok(())
}

/// # Errors
/// Returns `Err` when socket operations fail.
pub fn serve_udp(
    permit: &permit::Permit,
    sock: &std::net::UdpSocket,
    records: &[DnsRecord],
) -> Result<(), String> {
    sock.set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|e| format!("error setting socket read timeout: {}", e))?;
    let local_addr = sock
        .local_addr()
        .map_err(|e| format!("error getting socket local address: {}", e))?;
    let name_to_record: HashMap<&DnsName, &DnsRecord> =
        HashMap::from_iter(records.iter().map(|x| (x.name(), x)));
    while !permit.is_revoked() {
        // > Messages carried by UDP are restricted to 512 bytes (not counting the IP
        // > or UDP headers).  Longer messages are truncated and the TC bit is set in
        // > the header.
        // https://datatracker.ietf.org/doc/html/rfc1035#section-4.2.1
        let mut buf: FixedBuf<512> = FixedBuf::new();
        let addr = match sock.recv_from(buf.writable()) {
            // Can this happen?  The docs are not clear.
            Ok((len, _)) if len > buf.writable().len() => continue,
            Ok((len, addr)) => {
                buf.wrote(len);
                addr
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                continue
            }
            Err(e) => return Err(format!("error reading socket {:?}: {}", local_addr, e)),
        };
        let mut out: FixedBuf<512> = FixedBuf::new();
        if process_datagram(&name_to_record, buf, &mut out).is_err() {
            continue;
        }
        if out.is_empty() {
            unreachable!();
        }
        let sent_len = sock
            .send_to(out.readable(), &addr)
            .map_err(|e| format!("error sending response to {:?}: {}", addr, e))?;
        if sent_len != out.len() {
            return Err(format!(
                "sent only {} bytes of {} byte response to {:?}",
                sent_len,
                out.len(),
                addr
            ));
        }
    }
    Ok(())
}

//! safe-dns
//! ========
//! [![crates.io version](https://img.shields.io/crates/v/safe-dns.svg)](https://crates.io/crates/safe-dns)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/safe-dns/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! A threaded DNS server library.
//!
//! # Use Cases
//! - Make your API server its own DNS server.
//!   This eliminates the DNS server as a separate point of failure.
//! - Keep your DNS config in code, next to your server code.
//!   Include it in code reviews and integration tests.
//! - DNS-based
//!   [domain validation for free ACME certificates](https://letsencrypt.org/how-it-works/).
//!   This is useful for servers that don't listen on port 80.
//!   Servers on port 80 can use HTTP for domain validation and don't need to use this.
//!
//! # Features
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//! - ?% test coverage
//!
//! # Limitations
//! - Brand new.
//!
//! # Example
//!
//! # Related Crates
//!
//! # Cargo Geiger Safety Report
//! # Changelog
//! - v0.1.0 - Initial version
//!
//! # To Do
//! - Message compression
//! - Decide whether to send back error responses.
//! - Ergonomic constructors that take `OsStr`, for using environment variables
//! - Custom TTLs
//! - NS records (and glue)
//! - Client
//! - Caching client
//! - Recursive resolver
//!
//! # Alternatives
//!
#![forbid(unsafe_code)]

mod dns_class;
mod dns_message;
mod dns_message_header;
mod dns_name;
mod dns_op_code;
mod dns_question;
mod dns_record;
mod dns_response_code;
mod dns_server;
mod dns_type;

pub use dns_class::DnsClass;
pub use dns_message::DnsMessage;
pub use dns_message_header::DnsMessageHeader;
pub use dns_name::DnsName;
pub use dns_op_code::DnsOpCode;
pub use dns_question::DnsQuestion;
pub use dns_record::DnsRecord;
pub use dns_response_code::DnsResponseCode;
pub use dns_server::{process_datagram, serve_udp};
pub use dns_type::DnsType;

use fixed_buffer::FixedBuf;

fn read_exact<const N: usize, const M: usize>(buf: &mut FixedBuf<N>) -> Result<[u8; M], DnsError> {
    let mut result = [0_u8; M];
    buf.try_read_exact(&mut result).ok_or(DnsError::Truncated)?;
    Ok(result)
}

fn read_u8<const N: usize>(buf: &mut FixedBuf<N>) -> Result<u8, DnsError> {
    buf.try_read_byte().ok_or(DnsError::Truncated)
}

// fn write_u8<const N: usize>(out: &mut FixedBuf<N>, value: u8) -> Result<(), DnsError> {
//     out.write_bytes(&[value])
//         .map_err(|_| DnsError::ResponseBufferFull)?;
//     Ok(())
// }

fn read_u16_be<const N: usize>(buf: &mut FixedBuf<N>) -> Result<u16, DnsError> {
    let bytes: [u8; 2] = read_exact(buf)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u32_be<const N: usize>(buf: &mut FixedBuf<N>) -> Result<u32, DnsError> {
    let bytes: [u8; 4] = read_exact(buf)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn write_bytes<const N: usize>(out: &mut FixedBuf<N>, bytes: &[u8]) -> Result<(), DnsError> {
    out.write_bytes(bytes)
        .map_err(|_| DnsError::ResponseBufferFull)?;
    Ok(())
}

fn write_u16_be<const N: usize>(out: &mut FixedBuf<N>, value: u16) -> Result<(), DnsError> {
    let bytes: [u8; 2] = value.to_be_bytes();
    out.write_bytes(&bytes)
        .map_err(|_| DnsError::ResponseBufferFull)?;
    Ok(())
}

fn write_u32_be<const N: usize>(out: &mut FixedBuf<N>, value: u32) -> Result<(), DnsError> {
    let bytes: [u8; 4] = value.to_be_bytes();
    out.write_bytes(&bytes)
        .map_err(|_| DnsError::ResponseBufferFull)?;
    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DnsError {
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
    Internal(String),
    Unreachable(&'static str, u32),
}

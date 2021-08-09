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
//!
//! ## Features
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//! - ?% test coverage
//!
//! ## Limitations
//!
//! ## Alternatives
//!
//! ## Related Crates
//!
//! ## Example
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.0 - Initial version
#![forbid(unsafe_code)]

use core::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DnsName(String);
impl DnsName {
    fn err(value: impl AsRef<str>) -> Result<Self, String> {
        Err(format!("not a valid DNS name: {:?}", value.as_ref()))
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
        value.split(".").all(Self::is_valid_label)
    }

    pub fn new(value: impl AsRef<str>) -> Result<Self, String> {
        // Name syntax: https://datatracker.ietf.org/doc/html/rfc1035#page-8
        let mut trimmed = value.as_ref();
        trimmed = if trimmed.ends_with(".") {
            &trimmed[..(trimmed.len() - 1)]
        } else {
            trimmed
        };
        if !Self::is_valid_name(trimmed.as_ref()) {
            return Self::err(value);
        }
        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}
impl Display for DnsName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_name_separators() {
        // Separators.
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \".\"".to_string()),
            DnsName::new(".")
        );
        assert_eq!("a", DnsName::new("a.").unwrap().inner());
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"a..\"".to_string()),
            DnsName::new("a..")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \".a\"".to_string()),
            DnsName::new(".a")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"b..a\"".to_string()),
            DnsName::new("b..a")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \".b.a\"".to_string()),
            DnsName::new(".b.a")
        );
        // Labels.
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"\"".to_string()),
            DnsName::new("")
        );
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
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"1\"".to_string()),
            DnsName::new("1")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"1a\"".to_string()),
            DnsName::new("1a")
        );
        assert_eq!("a0", DnsName::new("a0").unwrap().inner());
        assert_eq!("a1", DnsName::new("a1").unwrap().inner());
        assert_eq!("a9", DnsName::new("a9").unwrap().inner());
        assert_eq!("a9876543210", DnsName::new("a9876543210").unwrap().inner());
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"-\"".to_string()),
            DnsName::new("-")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"a-\"".to_string()),
            DnsName::new("a-")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"-a\"".to_string()),
            DnsName::new("-a")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"a-.b\"".to_string()),
            DnsName::new("a-.b")
        );
        assert_eq!(
            <Result<DnsName, String>>::Err("not a valid DNS name: \"a.-b\"".to_string()),
            DnsName::new("a.-b")
        );
        assert_eq!("a-b", DnsName::new("a-b").unwrap().inner());
        assert_eq!("a-0", DnsName::new("a-0").unwrap().inner());
        assert_eq!("a---b", DnsName::new("a---b").unwrap().inner());
        assert_eq!(
            "xyz321-654abc",
            DnsName::new("Xyz321-654abC").unwrap().inner()
        );
    }
}

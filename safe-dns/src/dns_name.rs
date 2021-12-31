use crate::{read_byte, ProcessError};
use core::convert::TryFrom;
use core::fmt::{Display, Formatter};
use fixed_buffer::FixedBuf;

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
// TODO: Split this up into separate tests.
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

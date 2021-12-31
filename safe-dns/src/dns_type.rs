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
// TODO: Rename to DnsType.
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

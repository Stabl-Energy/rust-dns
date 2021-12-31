use crate::{DnsName, DnsType};
use core::fmt::{Debug, Formatter};
use std::net::IpAddr;

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

    pub fn typ(&self) -> DnsType {
        match self {
            DnsRecord::A(_, _) => DnsType::A,
            DnsRecord::AAAA(_, _) => DnsType::AAAA,
            DnsRecord::CNAME(_, _) => DnsType::CNAME,
        }
    }
}
impl Debug for DnsRecord {
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

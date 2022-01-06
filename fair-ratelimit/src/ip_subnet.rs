use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Copied from unstable `std::net::ip::Ipv4Addr::is_shared`.
fn is_shared4(addr: Ipv4Addr) -> bool {
    addr.octets()[0] == 100 && (addr.octets()[1] & 0b1100_0000 == 0b0100_0000)
}

/// Copied from unstable `std::net::ip::Ipv4Addr::is_reserved`.
fn is_reserved4(addr: Ipv4Addr) -> bool {
    addr.octets()[0] & 240 == 240 && !addr.is_broadcast()
}

/// Copied from unstable `std::net::ip::Ipv4Addr::is_benchmarking`.
fn is_benchmarking4(addr: Ipv4Addr) -> bool {
    addr.octets()[0] == 198 && (addr.octets()[1] & 0xfe) == 18
}

/// Copied from unstable `std::net::ip::Ipv4Addr::is_global`.
fn is_global4(addr: Ipv4Addr) -> bool {
    // check if this address is 192.0.0.9 or 192.0.0.10. These addresses are the only two
    // globally routable addresses in the 192.0.0.0/24 range.
    if u32::from_be_bytes(addr.octets()) == 0xc000_0009
        || u32::from_be_bytes(addr.octets()) == 0xc000_000a
    {
        return true;
    }
    !addr.is_private()
        && !addr.is_loopback()
        && !addr.is_link_local()
        && !addr.is_broadcast()
        && !addr.is_documentation()
        && !is_shared4( addr)
        // addresses reserved for future protocols (`192.0.0.0/24`)
        && !(addr.octets()[0] == 192 && addr.octets()[1] == 0 && addr.octets()[2] == 0)
        && !is_reserved4(addr)
        && !is_benchmarking4( addr)
        // Make sure the address is not in 0.0.0.0/8
        && addr.octets()[0] != 0
}

/// Copied from unstable `std::net::ip::Ipv6Addr::is_documentation`.
fn is_documentation6(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] == 0x2001) && (addr.segments()[1] == 0xdb8)
}

/// Copied from unstable `std::net::ip::Ipv6Addr::multicast_scope`.
const fn is_multicast_scope6(addr: Ipv6Addr) -> bool {
    if addr.is_multicast() {
        match addr.segments()[0] & 0x000f {
            1 => true,
            2 => true,
            3 => true,
            4 => true,
            5 => true,
            8 => true,
            14 => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Copied from unstable `std::net::ip::Ipv6Addr::is_unicast`.
fn is_unicast6(addr: Ipv6Addr) -> bool {
    !addr.is_multicast()
}

/// Copied from unstable `std::net::ip::Ipv6Addr::is_unicast_global`.
fn is_unicast_link_local6(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

/// Copied from unstable `std::net::ip::Ipv6Addr::is_unique_local`.
fn is_unique_local6(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

/// Copied from unstable `std::net::ip::Ipv6Addr::is_unicast_global`.
fn is_unicast_global6(addr: Ipv6Addr) -> bool {
    is_unicast6(addr)
        && !addr.is_loopback()
        && !is_unicast_link_local6(addr)
        && !is_unique_local6(addr)
        && !addr.is_unspecified()
        && !is_documentation6(addr)
}

/// Copied from unstable `std::net::ip::Ipv6Addr::is_global`.
fn is_global6(addr: Ipv6Addr) -> bool {
    is_multicast_scope6(addr) || is_unicast_global6(addr)
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IpSubnet {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}
impl IpSubnet {
    #[must_use]
    pub fn new4(addr: Ipv4Addr) -> Self {
        if is_global4(addr) {
            let mut octets = addr.octets();
            // Keep the /24 network part and zero out the rest of the address.
            octets[3] = 0;
            Self::Ipv4(Ipv4Addr::from(octets))
        } else {
            Self::Ipv4(addr)
        }
    }

    #[must_use]
    pub fn new6(addr: Ipv6Addr) -> Self {
        if addr.to_ipv4().is_some() {
            Self::Ipv6(addr)
        } else if is_global6(addr) {
            let mut segments = addr.segments();
            // Keep the /48 network part and zero out the rest of the address.
            segments[3] = 0;
            segments[4] = 0;
            segments[5] = 0;
            segments[6] = 0;
            segments[7] = 0;
            Self::Ipv6(Ipv6Addr::from(segments))
        } else {
            Self::Ipv6(addr)
        }
    }

    #[must_use]
    pub fn new(ip_addr: IpAddr) -> Self {
        match ip_addr {
            IpAddr::V4(addr) => Self::new4(addr),
            IpAddr::V6(addr) => Self::new6(addr),
        }
    }
}
impl From<IpAddr> for IpSubnet {
    fn from(addr: IpAddr) -> Self {
        Self::new(addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test4() {
        const ADDR4: Ipv4Addr = Ipv4Addr::new(11, 22, 33, 44);
        const SUBNET4: Ipv4Addr = Ipv4Addr::new(11, 22, 33, 0);
        assert_eq!(IpSubnet::Ipv4(SUBNET4), IpSubnet::new4(ADDR4));
        assert_eq!(IpSubnet::Ipv4(SUBNET4), IpSubnet::new(IpAddr::V4(ADDR4)));
        let subnet: IpSubnet = IpAddr::V4(ADDR4).into();
        assert_eq!(IpSubnet::Ipv4(SUBNET4), subnet);
        const LOOP4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
        assert_eq!(IpSubnet::Ipv4(LOOP4), IpSubnet::new4(LOOP4));
        const LOCAL4: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
        assert_eq!(IpSubnet::Ipv4(LOCAL4), IpSubnet::new4(LOCAL4));
    }

    #[test]
    fn test6() {
        const ADDR6: Ipv6Addr = Ipv6Addr::new(1111, 2222, 3333, 4, 5, 6, 7, 8);
        const SUBNET6: Ipv6Addr = Ipv6Addr::new(1111, 2222, 3333, 0, 0, 0, 0, 0);
        assert_eq!(IpSubnet::Ipv6(SUBNET6), IpSubnet::new6(ADDR6));
        assert_eq!(IpSubnet::Ipv6(SUBNET6), IpSubnet::new(IpAddr::V6(ADDR6)));
        let subnet: IpSubnet = IpAddr::V6(ADDR6).into();
        assert_eq!(IpSubnet::Ipv6(SUBNET6), subnet);
        const LOOP6: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(IpSubnet::Ipv6(LOOP6), IpSubnet::new6(LOOP6));
        const LOCAL6: Ipv6Addr = Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(IpSubnet::Ipv6(LOCAL6), IpSubnet::new6(LOCAL6));
    }
}

use crate::{read_exact, DnsError, DnsName, DnsType, ANY_CLASS, INTERNET_CLASS};
use fixed_buffer::FixedBuf;

#[derive(Debug, PartialEq)]
pub struct DnsQuestion {
    pub name: DnsName,
    pub typ: DnsType,
}
impl DnsQuestion {
    pub fn parse<const N: usize>(mut buf: FixedBuf<N>) -> Result<Self, DnsError> {
        let name = DnsName::read(&mut buf)?;
        let bytes: [u8; 4] = read_exact(&mut buf)?;
        let typ = DnsType::new(u16::from_be_bytes([bytes[0], bytes[1]]));
        let class = u16::from_be_bytes([bytes[2], bytes[3]]);
        if class != INTERNET_CLASS && class != ANY_CLASS {
            return Err(DnsError::InvalidClass);
        }
        Ok(DnsQuestion { name, typ })
    }
}

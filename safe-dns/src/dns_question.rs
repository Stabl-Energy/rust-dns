use crate::dns_class::DnsClass;
use crate::{DnsError, DnsName, DnsType};
use fixed_buffer::FixedBuf;

/// > The question section is used to carry the "question" in most queries, i.e., the parameters
/// > that define what is being asked.  The section contains QDCOUNT (usually 1) entries, each of
/// > the following format:
/// >
/// > ```text
/// >                                 1  1  1  1  1  1
/// >   0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
/// > +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// > |                                               |
/// > /                     QNAME                     /
/// > /                                               /
/// > +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// > |                     QTYPE                     |
/// > +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// > |                     QCLASS                    |
/// > +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// > ```
#[derive(Debug, PartialEq)]
pub struct DnsQuestion {
    pub name: DnsName,
    pub typ: DnsType,
}
impl DnsQuestion {
    /// # Errors
    /// Returns an error when `buf` does not contain a valid question struct.
    pub fn read<const N: usize>(mut buf: FixedBuf<N>) -> Result<Self, DnsError> {
        let name = DnsName::read(&mut buf)?;
        let typ = DnsType::read(&mut buf)?;
        let class = DnsClass::read(&mut buf)?;
        if class != DnsClass::Internet && class != DnsClass::Any {
            return Err(DnsError::InvalidClass);
        }
        Ok(DnsQuestion { name, typ })
    }
}

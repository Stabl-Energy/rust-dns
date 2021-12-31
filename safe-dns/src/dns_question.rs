use crate::{DnsName, DnsType};

// TODO: Moving parsing & generating code to here.
#[derive(Debug, PartialEq)]
pub struct DnsQuestion {
    pub name: DnsName,
    pub typ: DnsType,
}

use crate::{DnsName, Type};

// TODO: Rename to DnsQuestion.
// TODO: Moving parsing & generating code to here.
#[derive(Debug, PartialEq)]
pub struct Question {
    pub name: DnsName,
    pub typ: Type,
}

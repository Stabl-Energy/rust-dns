/// > `OPCODE`  A four bit field that specifies kind of query in this message.
/// >         This value is set by the originator of a query and copied into
/// >         the response.  The values are:
/// > - `0` a standard query (`QUERY`)
/// > - `1` an inverse query (`IQUERY`)
/// > - `2` a server status request (`STATUS`)
/// > - `3-15` reserved for future use
///
/// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
// TODO: Rename to DnsOpCode.
#[derive(Debug, PartialEq)]
pub enum OpCode {
    Query,
    InverseQuery,
    Status,
    Reserved(u8),
}
impl OpCode {
    pub fn new(value: u8) -> Self {
        match value {
            0 => OpCode::Query,
            1 => OpCode::InverseQuery,
            2 => OpCode::Status,
            other => OpCode::Reserved(other),
        }
    }
    pub fn num(&self) -> u8 {
        match self {
            OpCode::Query => 0,
            OpCode::InverseQuery => 1,
            OpCode::Status => 2,
            OpCode::Reserved(other) => *other,
        }
    }
}

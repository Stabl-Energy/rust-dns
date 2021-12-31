/// > `RCODE` Response code - this 4 bit field is set as part of responses.  The values have the
/// > following interpretation:
/// > - `0` No error condition
/// > - `1` Format error - The name server was unable to interpret the query.
/// > - `2` Server failure - The name server was unable to process this query due to a problem with
/// >   the name server.
/// > - `3` Name Error - Meaningful only for responses from an authoritative name server, this code
/// >   signifies that the domain name referenced in the query does not exist.
/// > - `4` Not Implemented - The name server does not support the requested kind of query.
/// > - `5` Refused - The name server refuses to perform the specified operation for policy reasons.
/// >   For example, a name server may not wish to provide the information to the particular
/// >   requester, or a name server may not wish to perform a particular operation (e.g., zone
/// >    transfer) for particular data.
/// > - `6-15` Reserved for future use.
///
/// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
// TODO: Rename to DnsResponseCode.
#[derive(Debug, PartialEq)]
pub enum ResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserved(u8),
}
impl ResponseCode {
    pub fn new(value: u8) -> Self {
        match value {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            other => ResponseCode::Reserved(other),
        }
    }
    pub fn num(&self) -> u8 {
        match self {
            ResponseCode::NoError => 0,
            ResponseCode::FormatError => 1,
            ResponseCode::ServerFailure => 2,
            ResponseCode::NameError => 3,
            ResponseCode::NotImplemented => 4,
            ResponseCode::Refused => 5,
            ResponseCode::Reserved(other) => *other,
        }
    }
}

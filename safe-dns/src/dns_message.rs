use crate::{
    read_exact, write_u16_be, DnsName, DnsOpCode, DnsQuestion, DnsRecord, DnsResponseCode, DnsType,
    ProcessError, ANY_CLASS, INTERNET_CLASS,
};
use fixed_buffer::FixedBuf;
use std::convert::TryFrom;

// TODO: Move fields and code to DnsMessageHeader.
pub struct DnsMessage {
    /// > `ID` A 16 bit identifier assigned by the program that generates any kind of query.  This
    /// > identifier is copied the corresponding reply and can be used by the requester to match up
    /// > replies to outstanding queries.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub id: u16,
    /// > `QR` A one bit field that specifies whether this message is a query (`0`),
    /// > or a response (`1`).
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub is_response: bool,
    pub op_code: DnsOpCode,
    /// > `AA` Authoritative Answer - this bit is valid in responses, and specifies that the
    /// > responding name server is an authority for the domain name in question section.
    /// >
    /// > Note that the contents of the answer section may have multiple owner names because of
    /// > aliases.  The AA bit corresponds to the name which matches the query name, or the first
    /// > owner name in the answer section.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub authoritative_answer: bool,
    /// > `TC` TrunCation - specifies that this message was truncated due to length greater than
    /// > that permitted on the transmission channel.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub truncated: bool,
    /// > `RD` Recursion Desired - this bit may be set in a query and is copied into the response.
    /// > If RD is set, it directs the name server to pursue the query recursively.  Recursive query
    /// > support is optional.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub recursion_desired: bool,
    /// > `RA` Recursion Available - this be is set or cleared in a response, and denotes whether
    /// > recursive query support is available in the name server.
    ///
    /// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub recursion_available: bool,
    pub response_code: DnsResponseCode,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub name_servers: Vec<DnsRecord>,
    pub additional: Vec<DnsRecord>,
}
impl DnsMessage {
    pub fn parse<const N: usize>(mut buf: FixedBuf<N>) -> Result<Self, ProcessError> {
        // Header
        let bytes: [u8; 12] = read_exact(&mut buf)?;
        let id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let is_response = (bytes[2] >> 7) == 1;
        let op_code = DnsOpCode::new((bytes[2] >> 3) & 0xF);
        let authoritative_answer = ((bytes[2] >> 2) & 1) == 1;
        let truncated = ((bytes[2] >> 1) & 1) == 1;
        let recursion_desired = (bytes[2] & 1) == 1;
        let recursion_available = (bytes[3] >> 7) == 1;
        let response_code = DnsResponseCode::new(bytes[3] & 0xF);
        let question_count = u16::from_be_bytes([bytes[4], bytes[5]]);
        let answer_count = u16::from_be_bytes([bytes[6], bytes[7]]);
        let name_server_count = u16::from_be_bytes([bytes[8], bytes[9]]);
        let additional_count = u16::from_be_bytes([bytes[10], bytes[11]]);
        if answer_count != 0 {
            return Err(ProcessError::QueryHasAnswer);
        }
        if name_server_count != 0 {
            return Err(ProcessError::QueryHasNameServer);
        }
        if additional_count != 0 {
            return Err(ProcessError::QueryHasAdditionalRecords);
        }
        // Questions
        let mut questions = Vec::with_capacity(question_count as usize);
        for _ in 0..question_count {
            let name = DnsName::read(&mut buf)?;
            let bytes: [u8; 4] = read_exact(&mut buf)?;
            let typ = DnsType::new(u16::from_be_bytes([bytes[0], bytes[1]]));
            let class = u16::from_be_bytes([bytes[2], bytes[3]]);
            if class != INTERNET_CLASS && class != ANY_CLASS {
                return Err(ProcessError::InvalidClass);
            }
            questions.push(DnsQuestion { name, typ });
        }
        Ok(Self {
            id,
            is_response,
            op_code,
            authoritative_answer,
            truncated,
            recursion_desired,
            recursion_available,
            response_code,
            questions,
            answers: Vec::new(),
            name_servers: Vec::new(),
            additional: Vec::new(),
        })
    }

    pub fn write<const N: usize>(&self, out: &mut FixedBuf<N>) -> Result<(), ProcessError> {
        let bytes: [u8; 2] = self.id.to_be_bytes();
        out.write_bytes(&bytes)
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        let b = ((self.is_response as u8) << 7)
            & (self.op_code.num() << 3)
            & ((self.authoritative_answer as u8) << 2)
            & ((self.truncated as u8) << 1)
            & (self.recursion_desired as u8);
        out.write_bytes(&[b])
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        let b = ((self.recursion_available as u8) << 7) & self.response_code.num();
        out.write_bytes(&[b])
            .map_err(|_| ProcessError::ResponseBufferFull)?;
        for count in [
            u16::try_from(self.questions.len()).map_err(|_| ProcessError::TooManyQuestions)?,
            u16::try_from(self.answers.len()).map_err(|_| ProcessError::TooManyAnswers)?,
            u16::try_from(self.name_servers.len()).map_err(|_| ProcessError::TooManyNameServers)?,
            u16::try_from(self.additional.len()).map_err(|_| ProcessError::TooManyAdditional)?,
        ] {
            write_u16_be(out, count)?;
        }
        if !self.questions.is_empty() {
            unimplemented!();
        }
        for record in self
            .answers
            .iter()
            .chain(self.name_servers.iter())
            .chain(self.additional.iter())
        {
            record.name().write(out)?;
            write_u16_be(out, record.typ().num())?;
            write_u16_be(out, INTERNET_CLASS)?;
            write_u16_be(out, 300_u16)?;
            // write_u16_be(out, rdlen)?;
            // write rdata
            todo!();
        }
        Ok(())
    }
}

use crate::{
    read_exact, write_u16_be, DnsMessageHeader, DnsName, DnsQuestion, DnsRecord, DnsResponseCode,
    DnsType, ProcessError, ANY_CLASS, INTERNET_CLASS,
};
use fixed_buffer::FixedBuf;

pub struct DnsMessage {
    pub header: DnsMessageHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub name_servers: Vec<DnsRecord>,
    pub additional: Vec<DnsRecord>,
}
impl DnsMessage {
    pub fn parse<const N: usize>(mut buf: FixedBuf<N>) -> Result<Self, ProcessError> {
        let header = DnsMessageHeader::parse(buf)?;
        if header.answer_count != 0 {
            return Err(ProcessError::QueryHasAnswer);
        }
        if header.name_server_count != 0 {
            return Err(ProcessError::QueryHasNameServer);
        }
        if header.additional_count != 0 {
            return Err(ProcessError::QueryHasAdditionalRecords);
        }
        // Questions
        let mut questions = Vec::with_capacity(header.question_count as usize);
        for _ in 0..header.question_count {
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
            header,
            questions,
            answers: Vec::new(),
            name_servers: Vec::new(),
            additional: Vec::new(),
        })
    }

    pub fn write<const N: usize>(&self, out: &mut FixedBuf<N>) -> Result<(), ProcessError> {
        self.header.write(out)?;
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

    pub fn answer_response(&self, answer: DnsRecord) -> Self {
        Self {
            header: DnsMessageHeader {
                id: self.header.id,
                is_response: true,
                op_code: self.header.op_code,
                authoritative_answer: true,
                truncated: false,
                recursion_desired: self.header.recursion_desired,
                recursion_available: false,
                response_code: DnsResponseCode::NoError,
                question_count: 0,
                answer_count: 1,
                name_server_count: 0,
                additional_count: 0,
            },
            questions: Vec::new(),
            answers: vec![answer],
            name_servers: Vec::new(),
            additional: Vec::new(),
        }
    }

    pub fn error_response(&self, response_code: DnsResponseCode) -> Self {
        Self {
            header: DnsMessageHeader {
                id: self.header.id,
                is_response: true,
                op_code: self.header.op_code,
                authoritative_answer: true,
                truncated: false,
                recursion_desired: self.header.recursion_desired,
                recursion_available: false,
                response_code,
                question_count: 0,
                answer_count: 0,
                name_server_count: 0,
                additional_count: 0,
            },
            questions: Vec::new(),
            answers: Vec::new(),
            name_servers: Vec::new(),
            additional: Vec::new(),
        }
    }
}

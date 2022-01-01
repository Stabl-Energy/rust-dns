use crate::{DnsError, DnsMessageHeader, DnsQuestion, DnsRecord, DnsResponseCode};
use fixed_buffer::FixedBuf;

pub struct DnsMessage {
    pub header: DnsMessageHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub name_servers: Vec<DnsRecord>,
    pub additional: Vec<DnsRecord>,
}
impl DnsMessage {
    /// # Errors
    /// Returns an error when `buf` does not contain a valid message.
    pub fn read<const N: usize>(buf: FixedBuf<N>) -> Result<Self, DnsError> {
        let header = DnsMessageHeader::read(buf)?;
        if header.answer_count != 0 {
            return Err(DnsError::QueryHasAnswer);
        }
        if header.name_server_count != 0 {
            return Err(DnsError::QueryHasNameServer);
        }
        if header.additional_count != 0 {
            return Err(DnsError::QueryHasAdditionalRecords);
        }
        // Questions
        let mut questions = Vec::with_capacity(header.question_count as usize);
        for _ in 0..header.question_count {
            let question = DnsQuestion::read(buf)?;
            questions.push(question);
        }
        Ok(Self {
            header,
            questions,
            answers: Vec::new(),
            name_servers: Vec::new(),
            additional: Vec::new(),
        })
    }

    /// # Errors
    /// Returns an error when `buf` fills up.
    ///
    /// # Panics
    /// Panics when `questions` is not empty.
    pub fn write<const N: usize>(&self, out: &mut FixedBuf<N>) -> Result<(), DnsError> {
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
            record.write(out)?;
        }
        Ok(())
    }

    #[must_use]
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

    #[must_use]
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

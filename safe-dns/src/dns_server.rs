use crate::{DnsMessage, DnsName, DnsOpCode, DnsRecord, ProcessError};
use fixed_buffer::FixedBuf;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::iter::FromIterator;
use std::time::Duration;

/// # Errors
/// Returns `Err` when the request is malformed or the server is not configured to answer the
/// request.
pub fn process_request(
    name_to_record: &HashMap<&DnsName, &DnsRecord>,
    request: DnsMessage,
) -> Result<DnsMessage, ProcessError> {
    if request.header.is_response {
        return Err(ProcessError::NotARequest);
    }
    if request.header.op_code != DnsOpCode::Query {
        return Err(ProcessError::InvalidOpCode);
    }
    // NOTE: We only answer the first question.
    let question = request.questions.first().ok_or(ProcessError::NoQuestion)?;
    // TODO: Support multiple answers.
    // u16::try_from(self.questions.len()).map_err(|_| ProcessError::TooManyQuestions)?,
    let record = *name_to_record
        .get(&question.name)
        .ok_or(ProcessError::NotFound)?;
    if record.typ() != question.typ {
        return Err(ProcessError::NotFound);
    }
    Ok(request.answer_response(record.clone()))
}

/// # Errors
/// Returns `Err` when the request is malformed or the server is not configured to answer the
/// request.
pub fn process_datagram(
    name_to_record: &HashMap<&DnsName, &DnsRecord>,
    bytes: FixedBuf<512>,
) -> Result<FixedBuf<512>, ProcessError> {
    let request = DnsMessage::parse(bytes)?;
    let response = process_request(name_to_record, request)?;
    let mut out: FixedBuf<512> = FixedBuf::new();
    response.write(&mut out)?;
    Ok(out)
}

/// # Errors
/// Returns `Err` when socket operations fail.
pub fn serve_udp(
    permit: &permit::Permit,
    sock: &std::net::UdpSocket,
    records: &[DnsRecord],
) -> Result<(), String> {
    sock.set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|e| format!("error setting socket read timeout: {}", e))?;
    let local_addr = sock
        .local_addr()
        .map_err(|e| format!("error getting socket local address: {}", e))?;
    let name_to_record: HashMap<&DnsName, &DnsRecord> =
        HashMap::from_iter(records.iter().map(|x| (x.name(), x)));
    while !permit.is_revoked() {
        // > Messages carried by UDP are restricted to 512 bytes (not counting the IP
        // > or UDP headers).  Longer messages are truncated and the TC bit is set in
        // > the header.
        // https://datatracker.ietf.org/doc/html/rfc1035#section-4.2.1
        let mut buf: FixedBuf<512> = FixedBuf::new();
        let addr = match sock.recv_from(buf.writable()) {
            // Can this happen?  The docs are not clear.
            Ok((len, _)) if len > buf.writable().len() => continue,
            Ok((len, addr)) => {
                buf.wrote(len);
                addr
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                continue
            }
            Err(e) => return Err(format!("error reading socket {:?}: {}", local_addr, e)),
        };
        let out = match process_datagram(&name_to_record, buf) {
            Ok(buf) => buf,
            Err(_e) => continue,
        };
        if out.is_empty() {
            unreachable!();
        }
        let sent_len = sock
            .send_to(out.readable(), &addr)
            .map_err(|e| format!("error sending response to {:?}: {}", addr, e))?;
        if sent_len != out.len() {
            return Err(format!(
                "sent only {} bytes of {} byte response to {:?}",
                sent_len,
                out.len(),
                addr
            ));
        }
    }
    Ok(())
}

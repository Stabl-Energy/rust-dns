use crate::{DnsError, DnsMessage, DnsName, DnsOpCode, DnsRecord};
use fixed_buffer::FixedBuf;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::time::Duration;

/// # Errors
/// Returns `Err` when the request is malformed or the server is not configured to answer the
/// request.
pub fn process_request(
    name_to_record: &HashMap<&DnsName, &DnsRecord>,
    request: &DnsMessage,
) -> Result<DnsMessage, DnsError> {
    if request.header.is_response {
        return Err(DnsError::NotARequest);
    }
    if request.header.op_code != DnsOpCode::Query {
        return Err(DnsError::InvalidOpCode);
    }
    // NOTE: We only answer the first question.
    let question = request.questions.first().ok_or(DnsError::NoQuestion)?;
    // u16::try_from(self.questions.len()).map_err(|_| ProcessError::TooManyQuestions)?,
    let record = *name_to_record
        .get(&question.name)
        .ok_or(DnsError::NotFound)?;
    if record.typ() != question.typ {
        return Err(DnsError::NotFound);
    }
    request.answer_response(record.clone())
}

/// # Errors
/// Returns `Err` when the request is malformed or the server is not configured to answer the
/// request.
#[allow(clippy::implicit_hasher)]
pub fn process_datagram(
    name_to_record: &HashMap<&DnsName, &DnsRecord>,
    bytes: &mut FixedBuf<512>,
) -> Result<FixedBuf<512>, DnsError> {
    let request = DnsMessage::read(bytes)?;
    let response = process_request(name_to_record, &request)?;
    let mut out: FixedBuf<512> = FixedBuf::new();
    response.write(&mut out)?;
    Ok(out)
}

pub fn make_name_to_record(records: &[DnsRecord]) -> HashMap<&DnsName, &DnsRecord> {
    records.iter().map(|x| (x.name(), x)).collect()
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
    let name_to_record = make_name_to_record(records);
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
        let out = match process_datagram(&name_to_record, &mut buf) {
            Ok(buf) => buf,
            Err(e) => {
                println!("dropping bad request: {:?}", e);
                continue;
            }
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

#[cfg(test)]
#[test]
fn test_process_datagram() {
    // From https://courses.cs.duke.edu//fall16/compsci356/DNS/DNS-primer.pdf
    // with some changes:
    // - Set result authoritative bit.
    let mut buf: FixedBuf<512> = FixedBuf::new();
    buf.write_bytes(&[
        0xdb, 0x42, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77, 0x77,
        0x77, 0x0c, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74, 0x65, 0x72, 0x6e, 0x03,
        0x65, 0x64, 0x75, 0x00, 0x00, 0x01, 0x00, 0x01,
    ])
    .unwrap();
    let expected_response = [
        0xdb, 0x42, 0x85, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77, 0x77,
        0x77, 0x0c, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74, 0x65, 0x72, 0x6e, 0x03,
        0x65, 0x64, 0x75, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00,
        0x00, 0x02, 0x58, 0x00, 0x04, 0x9b, 0x21, 0x11, 0x44_u8,
    ];
    let records = [DnsRecord::new_a("www.northeastern.edu", "155.33.17.68").unwrap()];
    let response = process_datagram(&make_name_to_record(&records), &mut buf).unwrap();
    assert_eq!(expected_response, response.readable());
}

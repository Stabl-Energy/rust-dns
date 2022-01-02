use crate::{DnsError, DnsMessage, DnsName, DnsOpCode, DnsRecord, DnsType};
use fixed_buffer::FixedBuf;
use multimap::MultiMap;
use std::io::ErrorKind;
use std::time::Duration;

/// # Errors
/// Returns `Err` when the request is malformed or the server is not configured to answer the
/// request.
pub fn process_request(
    name_to_records: &MultiMap<&DnsName, &DnsRecord>,
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
    let records = name_to_records
        .get_vec(&question.name)
        .ok_or(DnsError::NotFound)?;
    if question.typ == DnsType::ANY {
        request.answer_response(records.iter().copied())
    } else {
        request.answer_response(
            records
                .iter()
                .filter(|record| record.typ() == question.typ)
                .copied(),
        )
    }
}

/// # Errors
/// Returns `Err` when the request is malformed or the server is not configured to answer the
/// request.
#[allow(clippy::implicit_hasher)]
pub fn process_datagram(
    name_to_records: &MultiMap<&DnsName, &DnsRecord>,
    bytes: &mut FixedBuf<512>,
) -> Result<FixedBuf<512>, DnsError> {
    //println!("process_datagram: bytes = {:?}", bytes.readable());
    let request = DnsMessage::read(bytes)?;
    //println!("process_datagram: request = {:?}", request);
    let response = process_request(name_to_records, &request)?;
    //println!("process_datagram: response = {:?}", response);
    let mut out: FixedBuf<512> = FixedBuf::new();
    response.write(&mut out)?;
    //println!("process_datagram: out = {:?}", out.readable());
    Ok(out)
}

pub fn make_name_to_records(records: &[DnsRecord]) -> MultiMap<&DnsName, &DnsRecord> {
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
    let name_to_records = make_name_to_records(records);
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
        let out = match process_datagram(&name_to_records, &mut buf) {
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
        0x9A, 0x9A, 1, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 97, 97, 97,
        0x07, 101, 120, 97, 109, 112, 108, 101, 0x03, 99, 111, 109, 0x00, 0x00, 0x01, 0x00, 0x01,
    ])
    .unwrap();
    let expected_response = [
        0x9A, 0x9A, 0x85, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 97, 97, 97,
        0x07, 101, 120, 97, 109, 112, 108, 101, 0x03, 99, 111, 109, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x03, 97, 97, 97, 0x07, 101, 120, 97, 109, 112, 108, 101, 0x03, 99, 111, 109, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0x2C, 0x00, 0x04, 10, 0, 0, 1_u8,
    ];
    let records = [DnsRecord::new_a("aaa.example.com", "10.0.0.1").unwrap()];
    let response = process_datagram(&make_name_to_records(&records), &mut buf).unwrap();
    assert_eq!(expected_response, response.readable());
}

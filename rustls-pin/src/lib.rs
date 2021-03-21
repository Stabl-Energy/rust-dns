//! [![crates.io version](https://img.shields.io/crates/v/rustls-pin.svg)](https://crates.io/crates/rustls-pin)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/rustls-pin/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # rustls-pin
//!
//! Server certificate pinning with `rustls`.
//!
//! ## Features
//! - Make a TLS connection to a server
//! - Check that the server is using an allowed certificate
//! - `forbid(unsafe_code)`
//!
//! ## Alternatives
//! - [rustls#227 Implement support for certificate pinning](https://github.com/ctz/rustls/issues/227)
//!
//! ## Example
//! ```
//! # let listener = std::net::TcpListener::bind(&("127.0.0.1", 0)).unwrap();
//! # let addr = listener.local_addr().unwrap();
//! # std::thread::spawn(move || listener.accept().unwrap());
//! # let server_cert1 = rustls::Certificate(Vec::new());
//! # let server_cert2 = rustls::Certificate(Vec::new());
//! let mut stream = rustls_pin::connect_pinned(
//!     addr,
//!     vec![server_cert1, server_cert2],
//! ).unwrap();
//! ```
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]

use rustls::ClientSession;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;

/// A struct for TLS clients to verify the server's certificate.
/// Implements certificate pinning.
/// It accepts the server's certificate if it is identical to any of the certificates in the struct.
///
/// The rustls library has an open issue to add something like this:
/// "Implement support for certificate pinning" <https://github.com/ctz/rustls/issues/227>
///
/// # Example
///
/// ```
/// use std::net::TcpStream;
/// use std::sync::Arc;
/// # let listener = std::net::TcpListener::bind(&("127.0.0.1", 0)).unwrap();
/// # let addr = listener.local_addr().unwrap();
/// # std::thread::spawn(move || listener.accept().unwrap());
/// # let server_cert1 = rustls::Certificate(Vec::new());
/// # let server_cert2 = rustls::Certificate(Vec::new());
/// use rustls_pin::{
///     arbitrary_dns_name,
///     PinnedServerCertVerifier
/// };
/// let mut tcp_stream =
///     TcpStream::connect(addr).unwrap();
/// let mut config = rustls::ClientConfig::new();
/// config.dangerous().set_certificate_verifier(
///     Arc::new(
///         PinnedServerCertVerifier::new(vec![
///             server_cert1,
///             server_cert2
///         ]),
///     )
/// );
/// let mut session = rustls::ClientSession::new(
///     &Arc::new(config),
///     arbitrary_dns_name().as_ref()
/// );
/// let mut stream = rustls::Stream::new(
///     &mut session, &mut tcp_stream);
/// ```
pub struct PinnedServerCertVerifier<T>
where
    T: AsRef<[rustls::Certificate]> + Send + Sync,
{
    certs: T,
}

impl<T> PinnedServerCertVerifier<T>
where
    T: AsRef<[rustls::Certificate]> + Send + Sync,
{
    pub fn new(certs: T) -> Self {
        Self { certs }
    }
}

impl<T> rustls::ServerCertVerifier for PinnedServerCertVerifier<T>
where
    T: AsRef<[rustls::Certificate]> + Send + Sync,
{
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        // If the server sends several certificates (a certificate chain), we expect
        // the leaf certificate to be first.
        let presented_cert = &presented_certs[0];
        for cert in self.certs.as_ref() {
            if presented_cert == cert {
                return Ok(rustls::ServerCertVerified::assertion());
            }
        }
        Err(rustls::TLSError::WebPKIError(webpki::Error::UnknownIssuer))
    }
}

/// An arbitrary `DNSName` struct, for passing to [`rustls::ClientSession::new`].
/// `PinnedServerCertVerifier` receives the value and ignores it.
#[must_use]
pub fn arbitrary_dns_name() -> webpki::DNSName {
    webpki::DNSNameRef::try_from_ascii_str("arbitrary1")
        .unwrap()
        .to_owned()
}

/// Make a TCP connection to `addr` and set up a TLS session.
///
/// The first time you try to write or read the returned stream,
/// `rustls` will do TLS negotiation.
/// TLS negotiation fails if the server provides a leaf cert
/// that is not in `certs`.
///
/// Ignores hostnames in certificates.
///
/// # Errors
/// Returns an error if it fails to open the TCP connection.
///
/// # Example
/// See example in [`rustls_pin`](index.html) crate docs.
pub fn connect_pinned(
    addr: impl ToSocketAddrs,
    certs: impl AsRef<[rustls::Certificate]> + Send + Sync + 'static,
) -> Result<rustls::StreamOwned<ClientSession, TcpStream>, std::io::Error> {
    let tcp_stream = std::net::TcpStream::connect(addr)?;
    let mut client_config = rustls::ClientConfig::new();
    client_config
        .dangerous()
        .set_certificate_verifier(Arc::new(PinnedServerCertVerifier::new(certs)));
    let session =
        rustls::ClientSession::new(&Arc::new(client_config), arbitrary_dns_name().as_ref());
    Ok(rustls::StreamOwned::new(session, tcp_stream))
}

#[cfg(test)]
mod tests {
    use crate::{arbitrary_dns_name, PinnedServerCertVerifier};
    use std::io::{Read, Write};
    use std::sync::Arc;

    // # Generate the key and certificate with:
    // $ echo -e '[req]\ndistinguished_name=dn\nx509_extensions=ext\n[dn]\n[ext]\nsubjectAltName=@alt_names\n[alt_names]\nDNS.1=localhost\nIP.1=127.0.0.1\nIP.2=::1' >localhost.openssl.cfg
    // $ openssl req -newkey rsa:2048 -new -nodes -x509 -days 36500 -outform der -out localhost.crt.der -keyout localhost.key.pem -subj '/CN=localhost' -config localhost.openssl.cfg
    // # The openssl binary on macOS ignores the '-keyform der' argument, so we:
    // $ openssl rsa -in localhost.key.pem -outform DER -out localhost.key.der
    // $ cat localhost.key.der |base64 |fold -w 80
    // $ cat localhost.crt.der |base64 |fold -w 80
    // $ echo -e '[req]\ndistinguished_name=dn\nx509_extensions=ext\n[dn]\n[ext]\nsubjectAltName=@alt_names\n[alt_names]\nDNS.1=othername' >othername.openssl.cfg
    // $ openssl req -newkey rsa:2048 -new -nodes -x509 -days 36500 -outform der -out othername.crt.der -keyout othername.key.pem -subj '/CN=othername' -config othername.openssl.cfg
    // $ cat othername.crt.der |base64 |fold -w 80
    // $ rm localhost.openssl.cfg localhost.crt.der localhost.key.pem localhost.key.der othername.openssl.cfg othername.crt.der othername.key.pem
    const LOCALHOST_KEY_DER_BASE64: &str = r"
MIIEpAIBAAKCAQEAon826GrF4/sMKDsvYELwfJv3nqbfC59IK95/0qhwqkC6V52rxhzFesaQ4vxGhIZP
7XcVvBYSm+dgtCGnJZrUodzuTv0RfBNV9WKtgt91wXWI1n9xo5kPLewq1N1pp/mQYtUMXRUwt4f55qZW
0dyleaYQ35U2EdRpxKJQOsHoK+4QpBMkYZc202b7rQ97T5YnFFvEEPcqL7cKvIe5iEfBacsVbU8vscsH
Bg9WOr3ms3o0Wcgza+ioX8UZGr6VTNK+K/XDAdjsE8PsXTx7sC/QmVMVEYzWbD16tVnGiIy2yHCRmaEQ
qshwi6Dtu8wdSkfKoGsLU/1yLjfdmYV+CZeqGQIDAQABAoIBAQCQA9VDCRZXpnCw2ztywgDnP0enWVeG
smVBVBHPPq+ThIhiDIBntaoj1QYl1dYdr/f4irk9mAZoHBltrAG6Z02aIvcmFE3BvFVLhGgo/CkeCy/6
grrRdl6ooY2YWJ9CWwFnRlCN0rD5h86oZ7W8iyQw+0grD8/631naBsy6No6xFwOpUGFCSfEBF6LOgNgt
JroQVANcbVuGBibUO8L9rdR5FRnjw88mrcxP63QksBR6SaYdw6M+D67BJB1IxEeB9LB/inb/AzlWIoXF
eD79w9KAXBLon8T5aHWb4n32aIAe8G4j8RhfMuIcon6wkNqXBHMWTS7S10s0xpgGIcuD8YWtAoGBANcR
D4qrLTH/X8FINIEw3hcoC/+1xYNeJcwnBnPOJL88JW+KpjVrbXjKRDxNrLs2jnl9a7xH1sqs8dc2QdZk
fZBdrrsYjtJNWvrscp5MJ0NkYKA/ypXY27PC7AbQnZ3f5To0hLPzoBPVM3D+1BISgwcOGyUTzwScTZvC
t+4XFUSTAoGBAMFsvUEjA4Idguy0/mtF/XhoHhZTIAWb1k5WzS1Wqj0Cg0vRqoeJe51cflg5qMzE/LAq
mXysclDai8rkFE4DXy1xfuJ1clAjyZzo0Uj0lQabFooK8WRfXga7CPRsdCguyvRLoCvKg6PLVGhAfGQg
T9HV0yUdVLKwGBCoOtjzG84jAoGAaCvbW0+OlKkduIFA7VK+QHklVra09OylYj2E4pL3OanoeB6wYy+l
1twiMRNulz/VwwL9LDWf1IvwmE2vlikWqNa3y+gZRcQyTVg6LHK2ke4M35IGjo573JaNvL9PmSjZ31eQ
75kR8IlUYWcNUbOlw8URYOQ3YgRTkx69+JU1uAECgYEAjcRHShCBp8I9jYRy3OkliDS3qKEwXSwE/NH9
+/cDO0g2N0Hq/QA1S/bY240XPU477lqquIgkGUK1JvXYM/2gqsv+tbhjGn3AbXLuwcwR1g+hi3fNyUVk
wwYe4BcFY9Y4BqnPMYlyxoBm0ypAgZp1JlTUNuWyiG0sljjXON+mR4UCgYBJB1cUcEvy/cy1klPZEFhN
OX/mbPO8fvvuCPCGI5tvmJ1KBUIw/nTH/3wwUrw9TylPWfkqUKc45D+rRnOli/3i1PVAmNmgRriZ0xgi
7gZOwn/CwjFYTRj8T8JviEmSU/2zGgIMxN0bjz+6uHBRN9mM3/fr74gULEHZnjAkpje1Eg==";
    const LOCALHOST_CERT_DER_BASE64: &str = r"
MIIC3TCCAcWgAwIBAgIJAIdXrENNHCHgMA0GCSqGSIb3DQEBCwUAMBQxEjAQBgNVBAMMCWxvY2FsaG9z
dDAgFw0yMTAzMjAxNzIyMTJaGA8yMTIxMDIyNDE3MjIxMlowFDESMBAGA1UEAwwJbG9jYWxob3N0MIIB
IjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAon826GrF4/sMKDsvYELwfJv3nqbfC59IK95/0qhw
qkC6V52rxhzFesaQ4vxGhIZP7XcVvBYSm+dgtCGnJZrUodzuTv0RfBNV9WKtgt91wXWI1n9xo5kPLewq
1N1pp/mQYtUMXRUwt4f55qZW0dyleaYQ35U2EdRpxKJQOsHoK+4QpBMkYZc202b7rQ97T5YnFFvEEPcq
L7cKvIe5iEfBacsVbU8vscsHBg9WOr3ms3o0Wcgza+ioX8UZGr6VTNK+K/XDAdjsE8PsXTx7sC/QmVMV
EYzWbD16tVnGiIy2yHCRmaEQqshwi6Dtu8wdSkfKoGsLU/1yLjfdmYV+CZeqGQIDAQABozAwLjAsBgNV
HREEJTAjgglsb2NhbGhvc3SHBH8AAAGHEAAAAAAAAAAAAAAAAAAAAAEwDQYJKoZIhvcNAQELBQADggEB
ADSdn2p25k4cMNJ1nhUxxG2MK+kHCRlQxoZIw/79dd2fWFoBf1sgSFqD27hNyyLsl8cRwo99zrALYc3H
+CxCeZdnEaNtT2H8vkFgQQoJSBd2eY1FwyCsv63XHq3r9o+br48cnSNzgQkkMRL2n3efb7LeYwZw6dk0
uBPE5k/DmSeo9RYi502ToqnKj48FdN1/4qG5gkPoA7I0zztIKHvdn0FusocE73r8N6z9xe7Rw994nbCj
8/udFAovQUOWJMi3rHcVDf939NexoolAfQ7n51i657sRl7urcqUVz6VINuuiX7kcqcjSwM/27OlP0pcZ
kiQkQ/ZZmI9u3k2pzfse1/U=";
    const OTHERNAME_CERT_DER_BASE64: &str = r"
MIICxTCCAa2gAwIBAgIJAMc8xH62zNDpMA0GCSqGSIb3DQEBCwUAMBQxEjAQBgNVBAMMCW90aGVybmFt
ZTAgFw0yMTAzMjAxNzIyNTFaGA8yMTIxMDIyNDE3MjI1MVowFDESMBAGA1UEAwwJb3RoZXJuYW1lMIIB
IjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4NQLHxvjMhLKgSZ6yupmh2yztZFA3bx5pqPz/1Or
99a0wBwXU7pRcFsIgvDyQ+W9EI62KTFUQ/yA3COAZg/8cvld/BxcViDpwOuSRpo7qBOr82v7lUiGbRIU
/n51rgfS3KlWFrnqhtA9SWbSZjc68Dc41Ox24weJCEDBwuVxulTOjXwJ2L2MhPMP7Rs07cGhBZ6MJLYG
zrHqxrlwZNwBjIxAT/GBEPfHBNMXA0CRKrA0IBgzDB7VUxR6RmGGhzBlxbbN93RmXbUNnYnpGjUJF4n8
rjLjfw4nfMaPuS3uosEgX7BieOCtryFdQx+Mm5CDEXIw+3hQeK1iFfe0gFhe6QIDAQABoxgwFjAUBgNV
HREEDTALgglvdGhlcm5hbWUwDQYJKoZIhvcNAQELBQADggEBALxOeky0x+aOvQApoBW5b/GkfxkyE1B+
9p/+E2wQ61XIbNJzsdA5gF7TZvDSvOLEkTnN71Giiyvp3cq1oK4BONn+c5NrXlRpbMQTWATPg+oPkIR/
VQHAD/8bOxoKQH4+FW/65NOKfz3pr2LadYeeUqJ9CtmR0bbE+eIQOTqSjgM6r0SxoGHOzrgwidnrtG6t
9SR9Jm0QqAYyzMjJRq91+ZVEl2Pmk6vdjjPsc9Tf1vVIPqEohOjmr5XkhxQ6SKkbYjq1w5XCuyloAOEe
eiWkBybS6W+yeZ9qSu4x5AYceRO07rEs9C+wc4mUBtCnwUHgIAuGbuIuBPgykWxtxtN4JY8=";

    fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
        base64::decode(s.split_ascii_whitespace().collect::<String>())
    }

    #[test]
    fn test() {
        // Start a server thread.
        let localhost_key = rustls::PrivateKey(base64_decode(LOCALHOST_KEY_DER_BASE64).unwrap());
        let localhost_cert = rustls::Certificate(base64_decode(LOCALHOST_CERT_DER_BASE64).unwrap());
        let othername_cert = rustls::Certificate(base64_decode(OTHERNAME_CERT_DER_BASE64).unwrap());
        let listener = std::net::TcpListener::bind(&("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let mut server_config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        server_config
            .set_single_cert(vec![localhost_cert.clone()], localhost_key)
            .unwrap();
        let server_config_arc = Arc::new(server_config);
        std::thread::spawn(move || loop {
            let (mut tcp_stream, _addr) = listener.accept().unwrap();
            let mut tls_session = rustls::ServerSession::new(&server_config_arc);
            let mut tls_stream = rustls::Stream::new(&mut tls_session, &mut tcp_stream);
            if let Err(e) = tls_stream.write_all(b"response1") {
                eprintln!("WARN server write error: {:?}", e);
            }
        });
        {
            // Make a request to the server, accepting either cert.
            let mut tcp_stream = std::net::TcpStream::connect(addr).unwrap();
            let mut client_config = rustls::ClientConfig::new();
            client_config.dangerous().set_certificate_verifier(Arc::new(
                PinnedServerCertVerifier::new(vec![othername_cert.clone(), localhost_cert.clone()]),
            ));
            let mut session =
                rustls::ClientSession::new(&Arc::new(client_config), arbitrary_dns_name().as_ref());
            let mut stream = rustls::Stream::new(&mut session, &mut tcp_stream);
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            assert_eq!("response1", &response);
        }
        {
            // Make a request to the server, accepting only `othername_cert`, expecting error.
            let mut tcp_stream = std::net::TcpStream::connect(addr).unwrap();
            let mut client_config = rustls::ClientConfig::new();
            client_config.dangerous().set_certificate_verifier(Arc::new(
                PinnedServerCertVerifier::new(vec![othername_cert.clone()]),
            ));
            let mut session =
                rustls::ClientSession::new(&Arc::new(client_config), arbitrary_dns_name().as_ref());
            let mut stream = rustls::Stream::new(&mut session, &mut tcp_stream);
            let mut response = String::new();
            match stream.read_to_string(&mut response) {
                Err(e) if e.to_string() == "invalid certificate: UnknownIssuer".to_string() => {}
                other => panic!("{:?}", other),
            };
        }
    }
}

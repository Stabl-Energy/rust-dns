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
//! - 100% test coverage
//!
//! ## How to Update Pinned Certificates
//!
//! Before switching the server to a new certificate, you need to upgrade the
//! clients to accept both the current certificate and the new one.
//!
//! If your users update their client software infrequently, you may need to
//! wait a long time before switching to a new certificate.
//!
//! You can change certificates frequently by having multiple pending 'new'
//! certificates.  Example:
//! - Server: cert1
//! - Client v1: cert1
//! - Client v2: cert1, cert2
//! - Client v3: cert1, cert2, cert3
//! - Server: cert2
//! - Client v4: cert2, cert3, cert4
//! - Server: cert3
//! - Client v5: cert3, cert4, cert5
//! - Server cert4
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
//! let mut response = String::new();
//! match std::io::Read::read_to_string(
//!     &mut stream, &mut response) {
//!     Ok(_) => {},
//!     Err(e) if &e.to_string() ==
//!         "invalid certificate: UnknownIssuer"
//!      => panic!("Update required."),
//!     Err(e) => {
//!         // panic!("{}", e)
//!     }
//! }
//! ```
//!
//! When the client software reads/writes the stream and gets an
//! `invalid certificate: UnknownIssuer` error,
//! it can assume that it is outdated.
//! It can tell the user to update.
//!
//! The rustls client terminates the TLS connection by sending the
//! 'bad certificate' reason to the server.
//! The server's stream read/write fails with:
//! `"Custom { kind: InvalidData, error: AlertReceived(BadCertificate) }"`.
//!
//! ## Alternatives
//! - [rustls#227 Implement support for certificate pinning](https://github.com/ctz/rustls/issues/227)
//!
//! ## Changelog
//! - v0.1.2
//!   - Add "How to Update Pinned Certificates" to docs.
//!   - Add error handling to example
//! - v0.1.1 - Increase test coverage
//! - v0.1.0 - Initial version
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
#[allow(clippy::missing_panics_doc)]
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

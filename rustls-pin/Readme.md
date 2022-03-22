rustls-pin
==========
[![crates.io version](https://img.shields.io/crates/v/rustls-pin.svg)](https://crates.io/crates/rustls-pin)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/rustls-pin/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

Server certificate pinning with `rustls`.

# Features
- Make a TLS connection to a server
- Check that the server is using an allowed certificate
- `forbid(unsafe_code)`
- 100% test coverage

# How to Update Pinned Certificates

Before switching the server to a new certificate, you need to upgrade the
clients to accept both the current certificate and the new one.

If your users update their client software infrequently, you may need to
wait a long time before switching to a new certificate.

You can change certificates frequently by having multiple pending 'new'
certificates.  Example:
- Server: cert1
- Client v1: cert1
- Client v2: cert1, cert2
- Client v3: cert1, cert2, cert3
- Server: cert2
- Client v4: cert2, cert3, cert4
- Server: cert3
- Client v5: cert3, cert4, cert5
- Server cert4

# Example
```rust
let mut stream = rustls_pin::connect_pinned(
    addr,
    vec![server_cert1, server_cert2],
).unwrap();
let mut response = String::new();
match std::io::Read::read_to_string(
    &mut stream, &mut response) {
    Ok(_) => {},
    Err(e) if &e.to_string() ==
        "invalid certificate: UnknownIssuer"
     => panic!("Update required."),
    Err(e) => {
        // panic!("{}", e)
    }
}
```

When the client software reads/writes the stream and gets an
`invalid certificate: UnknownIssuer` error,
it can assume that it is outdated.
It can tell the user to update.

The rustls client terminates the TLS connection by sending the
'bad certificate' reason to the server.
The server's stream read/write fails with:
`"Custom { kind: InvalidData, error: AlertReceived(BadCertificate) }"`.

# Alternatives
- [rustls#227 Implement support for certificate pinning](https://github.com/ctz/rustls/issues/227)

# Changelog
- v0.1.2
  - Add "How to Update Pinned Certificates" to docs.
  - Add error handling to example
- v0.1.1 - Increase test coverage
- v0.1.0 - Initial version

# TO DO
- Support certificates that [`webpki` crate rejects](https://github.com/ctz/rustls/issues/127).
  The code is already
  [written](https://github.com/paritytech/x509-signature/issues/4#issuecomment-691729509).
  The tests may be challening to write.

License: Apache-2.0

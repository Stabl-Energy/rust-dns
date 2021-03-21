[![crates.io version](https://img.shields.io/crates/v/rustls-pin.svg)](https://crates.io/crates/rustls-pin)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/rustls-pin/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# rustls-pin

Server certificate pinning with `rustls`.

## Features
- Make a TLS connection to a server
- Check that the server is using an allowed certificate
- `forbid(unsafe_code)`

## Alternatives
- [rustls#227 Implement support for certificate pinning](https://github.com/ctz/rustls/issues/227)

## Example
```rust
let mut stream = rustls_pin::connect_pinned(
    addr,
    vec![server_cert1, server_cert2],
).unwrap();
```

## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

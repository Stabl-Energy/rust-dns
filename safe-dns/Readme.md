[![crates.io version](https://img.shields.io/crates/v/safe-dns.svg)](https://crates.io/crates/safe-dns)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/safe-dns/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# safe-dns

A threaded DNS server library.

## Use Cases
- Make your API server its own DNS server.
  This eliminates the DNS server as a separate point of failure.
- Keep your DNS config in code, next to your server code.
  Include it in code reviews and integration tests.
- DNS-based
  [domain validation for free ACME certificates](https://letsencrypt.org/how-it-works/).
  This is useful for servers that don't listen on port 80.
  Servers on port 80 can use HTTP for domain validation and don't need to use this.

## Features
- Depends only on `std`
- `forbid(unsafe_code)`
- ?% test coverage

## Limitations

## Example

## Related Crates

## Cargo Geiger Safety Report

## Changelog
- v0.1.0 - Initial version

# To Do
- `DoS` mitigation
- Message compression
- Decide whether to send back error responses.
- Ergonomic constructors that take `OsStr`, for using environment variables
- Custom TTLs
- NS records (and glue)
- Client
- Caching client
- Recursive resolver

## Alternatives


License: Apache-2.0

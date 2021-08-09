[![crates.io version](https://img.shields.io/crates/v/safe-dns.svg)](https://crates.io/crates/safe-dns)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/safe-dns/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# safe-dns

A threaded DNS server library.

## Use Cases

## Features
- Depends only on `std`
- `forbid(unsafe_code)`
- ?% test coverage

## Limitations

## Alternatives

## Related Crates

## Example

## Cargo Geiger Safety Report
```

Metric output format: x/y
    x = unsafe code used by the build
    y = total unsafe code found in the crate

Symbols: 
    🔒  = No `unsafe` usage found, declares #![forbid(unsafe_code)]
    ❓  = No `unsafe` usage found, missing #![forbid(unsafe_code)]
    ☢️  = `unsafe` usage found

Functions  Expressions  Impls  Traits  Methods  Dependency

0/0        0/0          0/0    0/0     0/0      🔒  safe-dns 0.1.0

0/0        0/0          0/0    0/0     0/0    

```
## Changelog
- v0.1.0 - Initial version

License: Apache-2.0

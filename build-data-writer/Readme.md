[![crates.io version](https://img.shields.io/crates/v/build-data-writer.svg)](https://crates.io/crates/build-data-writer)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/build-data-writer/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# build-data-writer

Functions to to write `build-data.txt` from your
[`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
Read the file with the
[`build-data`](https://crates.io/crates/build-data) crate.

## Features
- Saves build-time data:
  - Timestamp
  - Date-time string
  - Hostname
  - git commit, branch, and dirtiness
  - rustc version
- `forbid(unsafe_code)`
- 100% test coverage

## Alternatives
- [`build-info`](https://crates.io/crates/build-info)
  - Mature & popular
  - Confusing API
  - Uses procedural macros

## Example
See [`build-data`](https://crates.io/crates/build-data) crate docs.

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

0/0        0/0          0/0    0/0     0/0      🔒  build-data-writer 0.1.2
1/1        44/90        2/2    0/0     0/0      ☢️  └── chrono 0.4.19
0/19       10/311       0/0    0/0     5/27     ☢️      ├── libc 0.2.91
0/0        0/0          0/0    0/0     0/0      ❓      ├── num-integer 0.1.44
0/0        4/10         0/0    0/0     0/0      ☢️      │   └── num-traits 0.2.14
0/0        4/10         0/0    0/0     0/0      ☢️      ├── num-traits 0.2.14
0/0        0/7          0/0    0/0     0/0      ❓      ├── rustc-serialize 0.3.24
1/1        218/218      0/0    0/0     0/0      ☢️      └── time 0.1.44
0/19       10/311       0/0    0/0     5/27     ☢️          ├── libc 0.2.91
0/0        0/7          0/0    0/0     0/0      ❓          └── rustc-serialize 0.3.24

2/21       276/636      2/2    0/0     5/27   

```
## Changelog
- v0.1.2 - Support older versions of git when getting branch name.
- v0.1.1 - Initial version

## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

[![crates.io version](https://img.shields.io/crates/v/build-data.svg)](https://crates.io/crates/build-data)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/build-data/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# build-data

Include build data in your program.

## Features
- Saves build-time data:
  - Git commit, branch, and dirtiness
  - Source modification date & time
  - Rustc version
  - Rust channel (stable, nightly, or beta)
- Does all of its work in your
  [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
- Sets environment variables.
  Use [`env!`](https://doc.rust-lang.org/core/macro.env.html) to use them
  in your program.
- No macros
- No runtime dependencies
- Light build dependencies
- `forbid(unsafe_code)`
- 100% test coverage

## Alternatives
- Environment variables that cargo sets for crates:
  - `CARGO_PKG_NAME`
  - `CARGO_PKG_VERSION`
  - `CARGO_BIN_NAME`
  - [others](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates)
- [`vergen`](https://crates.io/crates/vergen)
  - Mature & very popular
  - Good API, needs only `env!` to retrieve values
  - Excellent test coverage
  - Heavy build dependencies
- [`build-info`](https://crates.io/crates/build-info)
  - Mature
  - Confusing API
  - Uses procedural macros

# Example

```toml
// Cargo.toml
[dependencies]

[build-dependencies]
build-data = "0"
```

Add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
file next to your `Cargo.toml`.
Call [`build_data::set_*`](https://docs.rs/build-data/) functions to
set variables.
```rust
// build.rs

fn main() {
    build_data::set_GIT_BRANCH();
    build_data::set_GIT_COMMIT();
    build_data::set_GIT_DIRTY();
    build_data::set_SOURCE_TIMESTAMP();
    build_data::no_debug_rebuilds();
}
```

Use [`env!`](https://doc.rust-lang.org/core/macro.env.html) to access the
variables in your program:
```rust
// src/bin/main.rs
fn main() {
    // Built from branch=release
    // commit=a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
    // dirty=false
    // source_timestamp=2021-04-14T06:25:59+00:00
    println!("Built from branch={} commit={} dirty={} source_timestamp={}",
        env!("GIT_BRANCH"),
        env!("GIT_COMMIT"),
        env!("GIT_DIRTY"),
        env!("SOURCE_TIMESTAMP"),
    );
}
```

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

0/0        0/0          0/0    0/0     0/0      🔒  build-data 0.1.3
1/1        44/90        2/2    0/0     0/0      ☢️  ├── chrono 0.4.19
1/20       10/327       0/2    0/0     5/30     ☢️  │   ├── libc 0.2.117
0/0        0/0          0/0    0/0     0/0      ❓  │   ├── num-integer 0.1.44
0/0        4/10         0/0    0/0     0/0      ☢️  │   │   └── num-traits 0.2.14
0/0        4/10         0/0    0/0     0/0      ☢️  │   ├── num-traits 0.2.14
0/0        0/7          0/0    0/0     0/0      ❓  │   ├── rustc-serialize 0.3.24
0/0        0/5          0/0    0/0     0/0      ❓  │   ├── serde 1.0.136
1/1        218/218      0/0    0/0     0/0      ☢️  │   └── time 0.1.44
1/20       10/327       0/2    0/0     5/30     ☢️  │       ├── libc 0.2.117
0/0        0/7          0/0    0/0     0/0      ❓  │       └── rustc-serialize 0.3.24
0/0        0/0          0/0    0/0     0/0      🔒  ├── safe-lock 0.1.3
0/0        0/0          0/0    0/0     0/0      🔒  └── safe-regex 0.2.4
0/0        0/0          0/0    0/0     0/0      🔒      └── safe-regex-macro 0.2.3
0/0        0/0          0/0    0/0     0/0      🔒          ├── safe-proc-macro2 1.0.24
0/0        0/0          0/0    0/0     0/0      🔒          │   └── unicode-xid 0.2.2
0/0        0/0          0/0    0/0     0/0      🔒          └── safe-regex-compiler 0.2.4
0/0        0/0          0/0    0/0     0/0      🔒              ├── safe-proc-macro2 1.0.24
0/0        0/0          0/0    0/0     0/0      🔒              └── safe-quote 1.0.9
0/0        0/0          0/0    0/0     0/0      🔒                  └── safe-proc-macro2 1.0.24

3/22       276/657      2/4    0/0     5/30   

```
## Changelog
- v0.1.3 - Update docs.
- v0.1.2 - Rewrote based on
    [feedback](https://www.reddit.com/r/rust/comments/mqnbvw/)
    from r/rust.
- v0.1.1 - Update docs.
- v0.1.0 - Initial version

## To Do

## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

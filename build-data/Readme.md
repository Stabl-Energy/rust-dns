[![crates.io version](https://img.shields.io/crates/v/build-data.svg)](https://crates.io/crates/build-data)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/build-data/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# build-data

Include build data in your program.

## Features
- Saves build-time data:
  - Git commit, branch, and dirtiness
  - Date & time
  - Epoch time
  - Hostname
  - Rustc version
- Does all of its work in your
  [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
- No macros
- Depends only on `core::alloc`
- `forbid(unsafe_code)`
- 100% test coverage

## Alternatives
- [`build-info`](https://crates.io/crates/build-info)
  - Mature & popular
  - Confusing API
  - Uses procedural macros

# Example

```toml
// Cargo.toml
[dependencies]
build-data = "0"

[build-dependencies]
build-data-writer = "0"
```

Add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
file next to your `Cargo.toml`.
Call [`build_data_writer::write`](https://docs.rs/build-data-writer/latest/build_data_writer/fn.write.html)
to collect data and write it to the file.
```rust
// build.rs
use std::env;
use std::path::Path;

fn main() {
    build_data_writer::write(
        &Path::new(&env::var_os("OUT_DIR").unwrap())
        .join("build-data.txt")
    ).unwrap();
    build_data_writer::no_debug_rebuilds();
}
```

When you run `cargo build`, Cargo compiles and runs your `build.rs` which
writes the file:
```
// target/build-data.txt
GIT_BRANCH:release
GIT_COMMIT:a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
GIT_DIRTY:false
HOSTNAME:builder2
RUSTC_VERSION:rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
TIME:2021-04-14T06:25:59+00:00
TIME_SECONDS:1618381559
```

Include and parse the file in your program.
See [`include_str!`](https://doc.rust-lang.org/core/macro.include_str.html),
[`concat!`](https://doc.rust-lang.org/core/macro.concat.html),
[`env!`](https://doc.rust-lang.org/core/macro.env.html), and
[`build_data::BuildData::new`](https://docs.rs/build-data/latest/build_data/struct.BuildData.html#method.new).
```rust
// src/bin/main.rs
fn main() {
    let bd = build_data::BuildData::new(include_str!(
        concat!(env!("OUT_DIR"), "/build-data.txt")
    )).unwrap();
    // Built 2021-04-14T06:25:59+00:00 branch=release
    // commit=a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
    // host=builder2
    // rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
    log!("{}", bd);
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

0/0        0/0          0/0    0/0     0/0      🔒  build-data 0.1.0

0/0        0/0          0/0    0/0     0/0    

```
## Changelog
- v0.1.0 - Initial version

## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

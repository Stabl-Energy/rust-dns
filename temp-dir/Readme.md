[![crates.io version](https://img.shields.io/crates/v/temp-dir.svg)](https://crates.io/crates/temp-dir)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/temp-dir/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# temp-dir

Provides a `TempDir` struct.

## Features
- Makes a directory in a system temporary directory
- Recursively deletes the directory and its contents on drop
- Optional name prefix
- No dependencies
- `forbid(unsafe_code)`

## Limitations
- Not security-hardened.

## Alternatives
- [`test_dir`](https://crates.io/crates/test_dir)
  - Has a handy `TestDir` struct
  - Incomplete documentation
- [`temp_testdir`](https://crates.io/crates/temp_testdir)
  - Incomplete documentation
- [`mktemp`](https://crates.io/crates/mktemp)
  - Sets directory mode 0700 on unix
  - Contains `unsafe`
  - No readme or online docs

## Related Crates
- [`temp-file`](https://crates.io/crates/temp-file)

## Example
```rust
use temp_dir::TempDir;
let d = TempDir::new().unwrap();
// Prints "/tmp/t1a9b0".
println!("{:?}", d.path());
let f = d.child("file1");
// Prints "/tmp/t1a9b0/file1".
println!("{:?}", f);
std::fs::write(&f, b"abc").unwrap();
assert_eq!(
    "abc",
    std::fs::read_to_string(&f).unwrap(),
);
// Prints "/tmp/t1a9b1".
println!(
    "{:?}", TempDir::new().unwrap().path());
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

0/0        0/0          0/0    0/0     0/0      🔒  temp-dir 0.1.2

0/0        0/0          0/0    0/0     0/0    

```
## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

[![crates.io version](https://img.shields.io/crates/v/temp-file.svg)](https://crates.io/crates/temp-file)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/temp-file/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# temp-file

Provides a `TempFile` struct.

## Features
- Makes a file in a system temporary directory
- Deletes the file on drop
- Optional file name prefix
- Optional file contents
- No dependencies
- `forbid(unsafe_code)`

## Limitations
- Not security-hardened. See
  [Secure Programming for Linux and Unix HOWTO - 7.10. Avoid Race Conditions](https://tldp.org/HOWTO/Secure-Programs-HOWTO/avoid-race.html)
  and [`mkstemp`](https://linux.die.net/man/3/mkstemp).

## Alternatives
- [`test-temp-file`](https://crates.io/crates/test-temp-file)
  - Depends on crates which contain `unsafe`
  - Incomplete documentation
- [`temp_file_name`](https://crates.io/crates/temp_file_name)
  - Does not delete file
  - Usage is not straightforward.  Missing example.
- [`mktemp`](https://crates.io/crates/mktemp)
  - Sets file mode 0600 on unix
  - Contains `unsafe`
  - No readme or online docs

## Related Crates
- [`temp-dir`](https://crates.io/crates/temp-dir)

## Example
```rust
use temp_file::TempFile;
let t = TempFile::new()
  .unwrap()
  .with_contents(b"abc")
  .unwrap();
// Prints "/tmp/1a9b0".
println!("{:?}", t.path());
assert_eq!(
  "abc",
  std::fs::read_to_string(t.path()).unwrap(),
);
// Prints "/tmp/1a9b1".
println!(
    "{:?}", TempFile::new().unwrap().path());
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

0/0        0/0          0/0    0/0     0/0      🔒  temp-file 0.1.0

0/0        0/0          0/0    0/0     0/0    

```
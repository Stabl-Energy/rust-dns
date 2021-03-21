[![crates.io version](https://img.shields.io/crates/v/any-range.svg)](https://crates.io/crates/any-range)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/any-range/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# any-range

`AnyRange<T>` enum can hold any `Range*<T>` type.

## Use Cases
- Store any kind of range in a struct without adding a type parameter

## Features
- `no_std`, depends only on `core`
- `forbid(unsafe_code)`

## Limitations
- Uses more bytes than a plain `Range<T>`.
  The alignment of `T` determines how many extra bytes the enum uses.

## Alternatives
- [`anyrange`](https://crates.io/crates/anyrange)
  - Should be called `ToRange`
  - Doesn't support `RangeInclusive` or `RangeToInclusive`
  - Unmaintained

## Example
```rust
use any_range::AnyRange;
let range: AnyRange<u8> = (3..5).into();
assert!(range.contains(&3));
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

0/0        0/0          0/0    0/0     0/0      🔒  any-range 0.1.1

0/0        0/0          0/0    0/0     0/0    

```
## Changelog
- v0.1.1 - Update docs
- v0.1.0 - Initial version

## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

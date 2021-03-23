[![crates.io version](https://img.shields.io/crates/v/safe-lock.svg)](https://crates.io/crates/safe-lock)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/safe-lock/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

# safe-lock

A simple `SafeLock` struct.

## Use Cases
- Run tests sequentially
- Prevent concurrent operations on atomic values
- Prevent concurrent operations on data and systems outside the Rust runtime

## Features
- Const constructor
- Depends only on `std`
- `forbid(unsafe_code)`
- 100% test coverage

## Limitations
- Not a `Mutex<T>`.  Does not contain a value.
- Unoptimized.  Uses
  [`AtomicBool`](https://doc.rust-lang.org/core/sync/atomic/struct.AtomicBool.html)
  in a spinlock, not fast OS locks.
- Not a fair lock.  If multiple threads acquire the lock in loops,
  some may never acquire it.

## Alternatives
- [`rusty-fork`](https://crates.io/crates/rusty-fork)
  - Run tests in separate processes
- [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
  - Part of the Rust standard library: well reviewed, well tested, and well maintained.
  - Uses fast OS locks
  - Has no const constructor.  See [rust#66806](https://github.com/rust-lang/rust/issues/66806)
    and [const-eval#3](https://github.com/rust-lang/const-eval/issues/3).
    You can work around this with unstable
    [`core::lazy::OnceCell`](https://doc.rust-lang.org/core/lazy/struct.OnceCell.html)
    or various `unsafe` crates:
    [`lazy_static`](https://crates.io/crates/lazy_static),
    [`once_cell`](https://crates.io/crates/once_cell),
    [`lazycell`](https://crates.io/crates/lazycell), and
    [`conquer-once`](https://crates.io/crates/conquer-once).
- [`parking_lot`](https://crates.io/crates/parking_lot)
  - Well written code.
    Many hope that it will end up in the Rust standard library someday.
  - Contains plenty of `unsafe`
- [`try-lock`](https://crates.io/crates/try-lock)
  - Popular
  - No dependencies, `no_std`
  - Uses `unsafe`
- [`ruspiro-lock`](https://crates.io/crates/ruspiro-lock)
  - Sync and async locks
  - No dependencies, `no_std`
  - Uses `unsafe`
- [`flexible-locks`](https://crates.io/crates/flexible-locks)
  - Lots of `unsafe`
  - Uses fast OS locks
  - Unmaintained

## Related Crates
- [`safina-sync`](https://crates.io/crates/safina-sync)
  provides a safe async `Mutex`

## Example

Make some tests run sequentially so they don't interfere with each other:
```unknown
use safe_lock::SafeLock;
static LOCK: SafeLock = SafeLock::new();

[#test]
fn test1() {
    let _guard = LOCK.lock();
    // ...
}

[#test]
fn test2() {
    let _guard = LOCK.lock();
    // ...
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

0/0        0/0          0/0    0/0     0/0      🔒  safe-lock 0.1.2

0/0        0/0          0/0    0/0     0/0    

```
## Changelog
- v0.1.3 - Increase test coverage
- v0.1.2 - Use `Acquire` and `Release` ordering
- v0.1.1 - Update docs
- v0.1.0 - Initial version

## Happy Contributors 🙂
Fixing bugs and adding features is easy and fast.
Send us a pull request and we intend to:
- Always respond within 24 hours
- Provide clear & concrete feedback
- Immediately make a new release for your accepted change

License: Apache-2.0

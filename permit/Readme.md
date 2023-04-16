[![crates.io version](https://img.shields.io/crates/v/permit.svg)](https://crates.io/crates/permit)
[![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/permit/LICENSE)
[![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
[![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)

[`permit::Permit`](https://docs.rs/permit/latest/permit/struct.Permit.html)
is a struct for cancelling operations.

# Use Cases
- Graceful server shutdown
- Cancel operations that take too long
- Stop in-flight operations when revoking authorization

# Features
- Subordinate permits.
  Revoking a permit also revokes its subordinates, recursively.
- Drop a permit to revoke its subordinates, recursively.
- Wait for all subordinate permits to drop.
- Implements `Future`.  You can `await` a permit and return when it is revoked.
- Similar to Golang's [`context`](https://golang.org/pkg/context/)
- Depends only on `std`.
- `forbid(unsafe_code)`
- 100% test coverage

# Limitations
- Does not hold data values
- Allocates.  Uses [`alloc::sync::Arc`](https://doc.rust-lang.org/alloc/sync/struct.Arc.html).

# Alternatives
- [`async_ctx`](https://crates.io/crates/async_ctx)
  - Good API
  - Async only
- [`stopper`](https://crates.io/crates/stopper/)
  - Async only
- [`io-context`](https://crates.io/crates/io-context)
  - Holds [Any](https://doc.rust-lang.org/core/any/trait.Any.html) values
  - Unmaintained
- [`ctx`](https://crates.io/crates/ctx)
  - Holds an [Any](https://doc.rust-lang.org/core/any/trait.Any.html) value
  - API is a direct copy of Golang's
    [`context`](https://golang.org/pkg/context/),
    even where that doesn't make sense for Rust.
    For example, to cancel, one must copy the context and call
    a returned `Box<Fn>`.
  - Unmaintained

# Related Crates

# Example

Graceful shutdown:
```rust
let top_permit = permit::Permit::new();
// Start some worker threads.
for _ in 0..5 {
    let permit = top_permit.new_sub();
    std::thread::spawn(move || {
        while !permit.is_revoked() {
            // ...
        }
    });
}
wait_for_shutdown_signal();
// Revoke all thread permits and wait for them to
// finish and drop their permits.
top_permit
    .revoke()
    .wait_subs_timeout(Duration::from_secs(3))
    .unwrap();
```

# Cargo Geiger Safety Report
```

Metric output format: x/y
    x = unsafe code used by the build
    y = total unsafe code found in the crate

Symbols: 
    🔒  = No `unsafe` usage found, declares #![forbid(unsafe_code)]
    ❓  = No `unsafe` usage found, missing #![forbid(unsafe_code)]
    ☢️  = `unsafe` usage found

Functions  Expressions  Impls  Traits  Methods  Dependency

0/0        0/0          0/0    0/0     0/0      🔒  permit 0.2.1

0/0        0/0          0/0    0/0     0/0    

```
# Changelog
- v0.2.1 - Fix bug where `sleep` and `sleep_until` would sometimes not return early.
- v0.2.0
   - Rename `try_wait_for` to `wait_subs_timeout`
   - Rename `try_wait_until` to `wait_subs_deadline`
   - Replace spinlock with Condvar in `wait*` methods
   - Remove `wait`
   - Add `sleep` and `sleep_until`
- v0.1.5 - Implement `Debug`
- v0.1.4 - Fix [bug](https://gitlab.com/leonhard-llc/ops/-/issues/2)
  where `revoke()` and then `wait()` would not wait.
- v0.1.3
  - Don't keep or wake stale
    [`std::task::Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html) structs.
  - Eliminate race that causes unnecessary wake.
- v0.1.2 - Implement `Future`
- v0.1.1 - Make `revoke` return `&Self`
- v0.1.0 - Initial version

License: Apache-2.0

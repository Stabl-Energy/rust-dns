# build-data-test
This is an integration test for
[`build-data`](https://crates.io/crates/build-data) and
[`build_data_writer`](https://crates.io/crates/build-data-writer)

License: Apache-2.0
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

0/0        0/0          0/0    0/0     0/0      🔒  build-data-test 0.1.0

0/0        0/0          0/0    0/0     0/0    

```

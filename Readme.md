> [!CAUTION]
>
> This repo was part of the dependencies for the [old edge device (UED)](https://github.com/Stabl-Energy/Rust-SBC-Client).
> It is archived as it is no longer needed.

# NOTE

This repository is a fork of [`ops/dns-server`](https://gitlab.com/leonhard-llc/ops/-/tree/main/dns-server).

Modifications:

- Removed server implementation
- Removed server related tests
- Removed local dependency `prob-rate-limiter`
- Removed parsing of field `additional` on dns requests as this information is not needed
- Moved dependencies `multimap` and `permit`  to dev-dependencies

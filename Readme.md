# NOTE

This repository is a fork of [`ops/dns-server`](https://gitlab.com/leonhard-llc/ops/-/tree/main/dns-server).

Modifications:

- Removed server implementation
- Removed server related tests
- Removed local dependency `prob-rate-limiter`
- Removed parsing of field `additional` on dns requests as this information is not needed
- Moved dependencies `multimap` and `permit`  to dev-dependencies

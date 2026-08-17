# Local Patch

This directory is derived from the MIT-licensed crates.io package `ddc-hi` 0.4.1.
The upstream license is retained in `COPYING`, and `Cargo.toml.orig` preserves
the published manifest.

The local manifest changes only these dependencies:

- `mccs` from `^0.1.0` to `^0.2`
- `mccs-caps` from `^0.1.0` to `^0.2`
- `mccs-db` from `^0.1.1` to `^0.2`

The crate source, `ddc` 0.2 dependency, and platform backend constraints remain
identical to the published package. This removes the `serde_yaml` 0.7 security
advisory through the maintained `mccs-db` 0.2 dependency chain without changing
the application's DDC API.

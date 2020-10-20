# C API for hyper

This provides auxiliary pieces for a C API to use the hyper library.

## Building

The C API is part of the Rust library, but isn't compiled by default. Using `cargo`, it can be compiled with the following command:

```
cargo build --no-default-features --features ffi
```

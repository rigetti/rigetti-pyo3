# Rigetti PyO3

This crate defines a set of macros for creating [PyO3](https://pyo3.rs) bindings to an *existing* Rust crate.

That is, given Rust library crate `foo`, these macros can be used inside a crate `foo-python` to create Python bindings. This is *not* intended for creating a standalone Python library using Rust.

See [the docs](https://docs.rs/rigetti-pyo3) for more.

## A note on feature compatibility

If you want to use [PyO3's `abi3` feature](https://pyo3.rs/v0.21.2/features#abi3), you must *disable* this library's `time` feature (which is enabled by default).  This library provides an `abi3` feature you can enable in order to explicitly request `pyo3/abi3`, which will give a clearer error message in that case.

----

Rigetti PyO3 is licensed under the [Apache License 2.0](LICENSE).

# Rigetti PyO3

This crate defines a set of macros for creating [PyO3](https://pyo3.rs) bindings to an *existing* Rust crate.

That is, given Rust library crate `foo`, these macros can be used inside a crate `foo-python` to create Python bindings. This is *not* intended for creating a standalone Python library using Rust.

See [the docs](https://docs.rs/rigetti-pyo3) for more.

**Note:** The above link will be broken until the crate is published on crates.io. For now, clone locally and use `cargo doc --open` to read the docs instead.
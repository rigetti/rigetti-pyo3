# Rigetti PyO3

This repository has helper crates for use with [PyO3](https://pyo3.rs):

- The [`rigetti-pyo3`](./crates/rigetti-pyo3/README.md) crate
  has useful macros to augment PyO3. 
- The [`optipy`](./crates/optipy/README.md) procedural macro 
  can be used to strip `PyO3`-related attributes within a crate
  when using Cargo features to optionally generate Python bindings.
- The [`pyo3-opentelemetry`](./crates/opentelemetry/README.md) crate 
  provides a macro to propagate OpenTelemetry context from Python into Rust.
  - The [`pyo3-opentelemetry-macros`](./crates/opentelemetry-macros/Cargo.toml)
    crate provides procedural macros for use with `pyo3-opentelemetry`.
- The [`pyo3-tracing-subscriber`](./crates/tracing-subscriber/README.md)
  crate provides a `PyModule` that supports configuration and initialization
  of Rust tracing subscribers from Python.
  - The [`pyo3-tracing-subscriber-build`](./crates/tracing-subscriber-build/Cargo.toml)
    crate implements build script support for `pyo3-tracing-subscriber`.

----

The contents of this repository are licensed under the [Apache License 2.0](LICENSE).

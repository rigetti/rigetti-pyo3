# Rigetti PyO3

This repository has helper macros for use with [PyO3](https://pyo3.rs):

- The [`rigetti-pyo3`](./rigetti-pyo3/README.md) crate
  has useful macros to augment PyO3. 
- The [`optipy`](./optipy/README.md) procedural macro 
  can be used to strip `PyO3`-related attributes within a crate
  when using Cargo features to optionally generate Python bindings.

----

The contents of this repository are licensed under the [Apache License 2.0](LICENSE).

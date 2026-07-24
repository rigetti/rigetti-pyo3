# PyO3 OpenTelemetry

## Background

### What this is

pyo3_opentelemetry provides a macro to simply and easily instrument your PyO3 bindings so that OpenTelemetry contexts can be easily passed from a Python caller into a Rust library. The `#[pypropagate]` macro instruments your Rust functions for you so that the global Python OpenTelemetry context is shared across the FFI boundary.

### What this is not

* This (currently) does not support propagating an OpenTelemetry context from Rust into Python.
* This does not "magically" instrument Rust code. Without the `#[pypropagate]` attribute, Rust code is unaffected and will not attach the Python OpenTelemetry context.
* This does not facilitate the processing or collection of OpenTelemetry spans; you still need to initialize and flush  tracing providers and subscribers separately in Python and Rust. For more information, please see the respective OpenTelemetry documentation for [Python](https://opentelemetry.io/docs/instrumentation/python/) and [Rust](https://opentelemetry.io/docs/instrumentation/rust/).

## Usage

> For a complete functioning example, see the `examples/pyo3-opentelemetry-lib/src/lib.rs` example within this crate's repository.

From Rust:

```rust
use pyo3_opentelemetry::prelude::*;
use pyo3::prelude::*;
use tracing::instrument;

#[pypropagate]
#[pyfunction]
#[instrument]
fn my_function() {
  println!("span \"my_function\" is active and will share the Python OpenTelemetry context");
}

#[pymodule]
fn my_module(_py: Python, m: &PyModule) -> PyResult<()> {
   m.add_function(wrap_pyfunction!(my_function, m)?)?;
   Ok(())
}
```

These features require no Python code changes, however, [opentelemetry-api](https://pypi.org/project/opentelemetry-api/) must be installed.



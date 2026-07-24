# PyO3 Tracing Subscriber 

## Background

### What this is

`pyo3_tracing_subscriber` provides a `PyModule` that can be added to upstream `pyo3` extension modules in order to support the configuration and initialization of Rust tracing subscribers from Python.

### What this is not

* Any initialized tracing subscriber imported from your upstream package will _not_ collect traces from any other `pyo3` extension module. In other words, any `pyo3` extension module will need to separately export tracing configuration and context managers, which in turn must be separately initialized in order to capture Rust traces from respective `pyo3` extension modules.
* Currently, only three tracing subscriber layers are supported:
    * `tracing_subscriber::fmt` which writes traces to file (or stdout) in a human readable format.
    * `opentelemetry-stdout` which writes traces to file (or stdout) in OTLP format. Available only with the `layer-otel-otlp-file` feature.
    * `opentelemetry-otlp` which sends traces to an OpenTelemetry OTLP endpoint. Available only with the `layer-otel-otlp` feature.
* This does not propagate OpenTelemetry contexts from Python into Rust (or vice versa). Use the `pyo3-opentelemetry` crate for that feature.

## Usage

> For a complete functioning example, see the `examples/pyo3-opentelemetry-lib/src/lib.rs` example within this crate's repository.

Given a `pyo3` extension module named "my_module" that would like to expose the tracing subscriber configuration and context manager classes from "my_module._tracing_subscriber", from Rust:

```rust
use pyo3::prelude::*;

#[pymodule]
fn my_module(py: Python, m: &PyModule) -> PyResult<()> {
    // Add your own Python classes, functions and modules.

    pyo3_tracing_subscriber::add_submodule(
        "my_module",
        "_tracing_subscriber",
        py,
        m,
    )?;
    Ok(())
}
```

Then a user could initialize a tracing subscriber that logged to stdout from Python:

```python
import my_module
from my_module._tracing_subscriber import (
    GlobalTracingConfig,
    SimpleConfig,
    Tracing,
    subscriber,
)
from pyo3_opentelemetry_lib._tracing_subscriber.layers import file


def main():
    tracing_configuration = GlobalTracingConfig(
        export_process=SimpleConfig(
            subscriber=subscriber.Config(
                layer=file.Config()
            )
        )
    )
    with Tracing(config=config):
        result = my_module.example_function()
        my_module.other_example_function(result)

if __name__ == '__main__':
    main()
```

### Building Python Stub Files

Use the companion [`pyo3-tracing-subscriber-build`](https://crates.io/crates/pyo3-tracing-subscriber-build) crate to generate stub files from your build script. Add it as a **build dependency** (not a regular dependency) so that `pyo3` is not pulled into your build script binary.

In `Cargo.toml`:

```toml
[dependencies]
pyo3-tracing-subscriber = { version = "...", features = ["layer-otel-otlp"] }

[build-dependencies]
pyo3-tracing-subscriber-build = { version = "...", features = ["layer-otel-otlp"] }
pyo3-build-config = "0.27"
```

The features on `pyo3-tracing-subscriber-build` must match those on `pyo3-tracing-subscriber`.

In `build.rs`:

```rust
use pyo3_tracing_subscriber_build::write_stub_files;

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    let target_dir = std::path::Path::new("./my_module/_tracing_subscriber");
    std::fs::remove_dir_all(target_dir).unwrap_or_default();
    write_stub_files("my_module", "_tracing_subscriber", target_dir).unwrap();
}
```


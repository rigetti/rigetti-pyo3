# pyo3-tracing-subscriber-build

Build script support for [`pyo3-tracing-subscriber`](https://crates.io/crates/pyo3-tracing-subscriber).

This crate generates Python stub (`.pyi`) files for the Python module added by
`pyo3_tracing_subscriber::add_submodule`. Add it as a **build dependency** (not a regular
dependency) to avoid pulling `pyo3` into your build script binary.

## Usage

In `Cargo.toml`:

```toml
[build-dependencies]
pyo3-tracing-subscriber-build = { version = "0.1", features = ["layer-otel-otlp"] }
pyo3-build-config = "0.27"
```

In `build.rs`:

```rust
use pyo3_tracing_subscriber_build::write_stub_files;

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    let target_dir = std::path::Path::new("./my_package/_tracing_subscriber");
    std::fs::remove_dir_all(target_dir).unwrap_or_default();
    write_stub_files("my_package", "_tracing_subscriber", target_dir).unwrap();
}
```

## Features

- `layer-otel-otlp-file` — include stubs for the `otel_otlp_file` layer
- `layer-otel-otlp` — include stubs for the `otel_otlp` layer

These should match the features enabled on `pyo3-tracing-subscriber` in your regular dependencies.

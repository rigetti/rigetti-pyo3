// Copyright 2023 Rigetti Computing
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Covers correctness, suspicious, style, complexity, and perf
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![warn(clippy::nursery)]
// Has false positives that conflict with unreachable_pub
#![allow(clippy::redundant_pub_crate)]
#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    bad_style,
    dead_code,
    keyword_idents,
    improper_ctypes,
    macro_use_extern_crate,
    meta_variable_misuse, // May have false positives
    missing_abi,
    missing_debug_implementations, // can affect compile time/code size
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    noop_method_call,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    semicolon_in_expressions_from_macros,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_parens,
    variant_size_differences,
    while_true
)]
//! This crate provides utilities for configuring and initializing a tracing subscriber from
//! Python. Because Rust pyo3-based Python packages are binaries, these utilities are exposed
//! as a `pyo3::types::PyModule` which can then be added to upstream pyo3 libraries.
//!
//! # Features
//!
//! * `layer-otel-otlp-file` - exports trace data with `opentelemetry-stdout`. See `crate::layers::otel_otlp_file`.
//! * `layer-otel-otlp` - exports trace data with `opentelemetry-otlp`. See `crate::layers::otel_otlp`.
//!
//! # Requirements and Limitations
//!
//! * The tracing subscribers initialized and configured _only_ capture tracing data for the pyo3
//!   library which adds the `pyo3-tracing-subscriber` module. Separate Python libraries require separate
//!   bootstrapping.
//! * Python users can initialize tracing subscribers using context managers either globally, in
//!   which case they can only initialize once, or per-thread, which is incompatible with Python
//!   `async/await`.
//! * The `OTel` OTLP layer requires a heuristic based timeout upon context manager exit to ensure
//!   trace data on the Rust side is flushed to the OTLP collector. This issue currently persists despite calls
//!   to `force_flush` on the `opentelemetry_sdk::trace::TracerProvider` and `opentelemetry::global::shutdown_tracer_provider`.
//!
//! # Examples
//!
//! ```
//! use pyo3::prelude::*;
//! use tracing::instrument;
//!
//! const MY_PACKAGE_NAME: &str = "example";
//! const TRACING_SUBSCRIBER_SUBMODULE_NAME: &str = "tracing_subscriber";
//!
//! #[pymodule]
//! fn example(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     // add your functions, modules, and classes
//!     pyo3_tracing_subscriber::add_submodule(
//!         MY_PACKAGE_NAME,
//!         TRACING_SUBSCRIBER_SUBMODULE_NAME,
//!         py,
//!         m,
//!     )?;
//!     Ok(())
//! }
//! ```
//!
//! Then in Python:
//!
//! ```python
//! import asyncio
//! from example.tracing_subscriber import Tracing
//!
//!
//! async main():
//!     async with Tracing():
//!
//!      # do stuff
//!         pass
//!
//!
//! if __name__ == "__main__":
//!     asyncio.run(main())
//! ```
//!
//! # Related Crates
//!
//! * `pyo3-opentelemetry` - propagates `OpenTelemetry` contexts from Python into Rust.
use pyo3::{prelude::*, types::PyModule, PyResult, Python};
use rigetti_pyo3::create_init_submodule;

use self::{
    contextmanager::{CurrentThreadTracingConfig, GlobalTracingConfig, TracingContextManagerError},
    export_process::{BatchConfig, SimpleConfig, TracingShutdownError, TracingStartError},
};
pub use contextmanager::Tracing;

pub(crate) mod common;
mod contextmanager;
mod export_process;
pub(crate) mod layers;
pub(crate) mod subscriber;

create_init_submodule! {
    classes: [
        Tracing,
        GlobalTracingConfig,
        CurrentThreadTracingConfig,
        BatchConfig,
        SimpleConfig
    ],
    errors: [TracingContextManagerError, TracingStartError, TracingShutdownError],
    submodules: [
        "layers": layers::init_submodule,
        "subscriber": subscriber::init_submodule,
        "common": common::init_submodule
    ],
}

/// Add the tracing submodule to the given module. This will add the submodule to the `sys.modules`
/// dictionary so that it can be imported from Python.
///
/// # Arguments
///
/// * `fully_qualified_namespace` - the fully qualified namespace of the parent Python module to
///   which the tracing submodule should be added. This may be a nested namespace, such as
///   `my_package.my_module`.
/// * `name` - the name of the tracing subscriber submodule within the specified parent module.
///   This should not be a nested namespace.
/// * `py` - the Python GIL token.
/// * `parent_module` - the parent module to which the tracing subscriber submodule should be added.
///
/// # Errors
///
/// * `PyErr` if the submodule cannot be added.
///
/// # Additional Details
///
/// This function will add the following:
///
/// * `Tracing` - a Python context manager which initializes the configured tracing subscriber.
/// * `GlobalTracingConfig` - a Python context manager which sets the configured tracing subscriber
///   as the global default (ie `tracing::subscriber::set_global_default`). The `Tracing` context
///   manager can be used _only once_ per process with this configuration.
/// * `CurrentThreadTracingConfig` - a Python context manager which sets the configured tracing
///   subscriber as the current thread default (ie `tracing::subscriber::set_default`). As the
///   context manager exits, the guard is dropped and the tracing subscriber can be re-initialized
///   with another default. Note, the default tracing subscriber will _not_ capture traces across
///   `async/await` boundaries that call `pyo3_asyncio::tokio::future_into_py`.
/// * `BatchConfig` - a Python context manager which configures the tracing subscriber to export
///   trace data in batch. As the `Tracing` context manager enters, a Tokio runtime is initialized
///   and will run in the background until the context manager exits.
/// * `SimpleConfig` - a Python context manager which configures the tracing subscriber to export
///   trace data in a non-batch manner. This only initializes a Tokio runtime if the underlying layer
///   requires an asynchronous runtime to export trace data (ie the `opentelemetry-otlp` layer).
/// * `layers` - a submodule which contains different layers to add to the tracing subscriber.
///   Currently supported:
///     * `tracing::fmt` - a layer which exports trace data to stdout in a non-OpenTelemetry data format.
///     * `opentelemetry-stdout` - a layer which exports trace data to stdout (requires the `layer-otel-otlp-file` feature).
///     * `opentelemetry-otlp` - a layer which exports trace data to an `OpenTelemetry` collector (requires the `layer-otel-otlp` feature).
/// * `subscriber` - a submodule which contains utilities for initialing the tracing subscriber
///   with the configured layer. Currently, the tracing subscriber is initialized as
///   `tracing::subscriber::Registry::default().with(layer)`.
///
/// The following exceptions are added to the submodule:
///
/// * `TracingContextManagerError` - raised when the `Tracing` context manager's methods are not
///   invoked in the correct order or multiplicity.
/// * `TracingStartError` - raised if the user-specified tracing layer or subscriber fails to build
///   and initialize properly upon context manager entry.
/// * `TracingShutdownError` - raised if the tracing layer or subscriber fails to shutdown properly on context manager exit.
///
/// For detailed Python usage documentation, see the stub files written by
/// [`crate::stubs::write_stub_files`].
pub fn add_submodule<'py>(
    fully_qualified_namespace: &str,
    name: &str,
    py: Python<'py>,
    parent_module: &Bound<'py, PyModule>,
) -> PyResult<()> {
    let tracing_subscriber = PyModule::new(py, name)?;
    let fully_qualified_name = format!("{fully_qualified_namespace}.{name}");
    init_submodule(&fully_qualified_name, py, &tracing_subscriber)?;
    parent_module.add_submodule(&tracing_subscriber)?;
    let modules = py.import("sys")?.getattr("modules")?;
    modules.set_item(fully_qualified_name, tracing_subscriber)?;
    Ok(())
}

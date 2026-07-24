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
//! This crate provides a function for attaching an `OpenTelemetry` context from Python within
//! Rust. It is intended to be used in conjunction with the `pypropagate` macro, which is
//! re-exported here.
//!
//! # Requirements and Limitations
//!
//! * All functionality here requires the calling Python code to have [opentelemetry-api](https://pypi.org/project/opentelemetry-api/) to be installed.
//! * See `pypropagate` for additional requirements and limitations.
//!
//! # Related Crates
//!
//! * `pyo3-opentelemetry-macros` - a crate defining the `pypropagate` macro.
//! * `pyo3-tracing-subscriber` - a crate supporting configuration and initialization of Rust
//!   tracing subscribers from Python.
//!
//! # Examples
//!
//! ```
//! use pyo3_opentelemetry::pypropagate;
//! use pyo3::prelude::*;
//! use tracing::instrument;
//!
//! #[pypropagate]
//! #[pyfunction]
//! #[instrument(skip(py))]
//! fn my_function(py: Python<'_>) {
//!   println!("span \"my_function\" is active and will share the Python OpenTelemetry context");
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!    m.add_function(wrap_pyfunction!(my_function, m)?)?;
//!    Ok(())
//! }
//! ```
//!
//! For a more comprehensive example, see the `pyo3-opentelemetry-lib` example in this repository.
//! Specifically, see the `pyo3-opentelemetry-lib/src/lib.rs` for the Rust code and `pyo3-opentelemetry-lib/pyo3_opentelemetry_lib/tests/test_tracing.py` for the Python code and behavioural assertions.
use std::collections::HashMap;

use pyo3::{prelude::*, types::IntoPyDict};

use qcs_dependencies_client::{
    opentelemetry::{
        propagation::{Extractor, TextMapPropagator},
        Context, ContextGuard,
    },
    opentelemetry_sdk::propagation::TraceContextPropagator,
};

pub use pyo3_opentelemetry_macros::pypropagate;

#[doc(hidden)]
pub use qcs_dependencies_client::opentelemetry as __opentelemetry;

/// A context carrier for propagating `OpenTelemetry` context from Python to Rust.
#[derive(Default, Clone, Debug, FromPyObject)]
struct Carrier {
    /// The context [traceparent](https://www.w3.org/TR/trace-context/#traceparent-header).
    #[pyo3(item)]
    traceparent: Option<String>,
    /// The context [tracestate](https://www.w3.org/TR/trace-context/#tracestate-header).
    #[pyo3(item)]
    tracestate: Option<String>,
}

const TRACEPARENT_HEADER: &str = "traceparent";
const TRACESTATE_HEADER: &str = "tracestate";

impl Extractor for Carrier {
    fn get(&self, key: &str) -> Option<&str> {
        match key.to_lowercase().as_str() {
            TRACEPARENT_HEADER => self.traceparent.as_deref(),
            TRACESTATE_HEADER => self.tracestate.as_deref(),
            _ => None,
        }
    }

    fn keys(&self) -> Vec<&str> {
        vec![TRACEPARENT_HEADER, TRACESTATE_HEADER]
    }
}

impl From<HashMap<String, String>> for Carrier {
    fn from(value: HashMap<String, String>) -> Self {
        Self {
            tracestate: value.get(TRACESTATE_HEADER).cloned(),
            traceparent: value.get(TRACEPARENT_HEADER).cloned(),
        }
    }
}

impl Carrier {
    /// When a `Propagator` is passed to a function or method, this method should be called
    /// at the beginning of the function or method to attach the context. This should not be used with
    /// async functions.
    fn attach(&self) -> ContextGuard {
        let propagator = TraceContextPropagator::new();
        Context::attach(propagator.extract(self))
    }
}

/// Attach the current `OpenTelemetry` context from Python. This should be called at the beginning of
/// a function or method to attach the context. This should not be used with async functions.
///
/// # Requirements and Limitations
///
/// * The calling Python code must have [opentelemetry-api](https://pypi.org/project/opentelemetry-api/) installed.
/// * Use the `pypropagate` macro instead of this function directly.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3_opentelemetry::attach_otel_context_from_python;
///
/// fn my_function(py: Python<'_>) -> PyResult<()> {
///  let _guard = attach_otel_context_from_python(py)?;
///  println!("span \"my_function\" is active and will share the Python OpenTelemetry context");
///  // ...
///  Ok(())
/// }
/// ```
///
/// # Errors
///
/// Any Python error that occurs while trying to get the current context from Python will
/// be returned; this includes import errors when importing `opentelemetry.context` and
/// `opentelemetry.propagate`.
pub fn attach_otel_context_from_python(py: Python<'_>) -> PyResult<ContextGuard> {
    let get_current_context = py.import("opentelemetry.context")?.getattr("get_current")?;
    let inject = py.import("opentelemetry.propagate")?.getattr("inject")?;

    let current_context = get_current_context.call0()?;
    let data = pyo3::types::PyDict::new(py).into_any();
    let kwargs = [("context", current_context), ("carrier", data.clone())].into_py_dict(py)?;
    inject.call((), Some(&kwargs))?;

    let data: HashMap<String, String> = data.extract()?;
    let carrier: Carrier = data.into();

    Ok(carrier.attach())
}

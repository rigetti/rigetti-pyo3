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
//! Build script support for `pyo3-tracing-subscriber`.
//!
//! This crate provides a function to generate Python stub files for the Python module added by
//! `pyo3_tracing_subscriber::add_submodule`. Use this crate as a build dependency (not a regular
//! dependency) so that pyo3 is not pulled into your build script binary.
//!
//! # Features
//!
//! * `layer-otel-otlp-file` - include stubs for the `otel_otlp_file` layer.
//! * `layer-otel-otlp` - include stubs for the `otel_otlp` layer.
//!
//! # Example
//!
//! In `build.rs` with the `example/` directory containing Python source code:
//!
//! ```rust,no_run
//! use pyo3_tracing_subscriber_build::write_stub_files;
//!
//! write_stub_files(
//!     "example",
//!     "tracing_subscriber",
//!     std::path::Path::new("example/tracing_subscriber"),
//! ).unwrap();
//! ```

use std::path::Path;

use handlebars::{RenderError, TemplateError};

#[derive(serde::Serialize, Default)]
struct Data {
    host_package: String,
    tracing_subscriber_module_name: String,
    version: String,
    layer_otel_otlp_file: bool,
    layer_otel_otlp: bool,
    any_additional_layer: bool,
}

impl Data {
    fn new(host_package: String, tracing_subscriber_module_name: String) -> Self {
        Self {
            host_package,
            tracing_subscriber_module_name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            layer_otel_otlp_file: cfg!(feature = "layer-otel-otlp-file"),
            layer_otel_otlp: cfg!(feature = "layer-otel-otlp"),
            any_additional_layer: cfg!(feature = "layer-otel-otlp-file")
                || cfg!(feature = "layer-otel-otlp"),
        }
    }
}

/// Errors that may occur when writing stub files.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to open file for writing.
    #[error("failed open file for writing: {0}")]
    Io(#[from] std::io::Error),
    /// Failed to render template.
    #[error("failed to render template: {0}")]
    Render(#[from] RenderError),
    /// Failed to initialize template.
    #[error("failed to initialize template: {0}")]
    Template(#[from] Box<TemplateError>),
}

macro_rules! include_stub_and_init {
    ($directory: ident, $template_name: tt, $hb: ident) => {
        std::fs::create_dir_all($directory.join($template_name)).map_err(Error::from)?;
        $hb.register_template_string(
            concat!($template_name, "__init__.py"),
            include_str!(concat!(
                "../assets/python_stubs/",
                $template_name,
                "__init__.py"
            )),
        )
        .map_err(Box::new)
        .map_err(Error::from)?;
        $hb.register_template_string(
            concat!($template_name, "__init__.pyi"),
            include_str!(concat!(
                "../assets/python_stubs/",
                $template_name,
                "__init__.pyi"
            )),
        )
        .map_err(Box::new)
        .map_err(Error::from)?;
    };
}

/// Write stub files for the given host package and tracing subscriber module name to the given
/// directory.
///
/// # Arguments
///
/// * `host_package` - The name of the host Python package.
/// * `tracing_subscriber_module_name` - The name of the tracing subscriber module (i.e. the Python
///   module that will contain the stub files).
/// * `directory` - The directory to write the stub files to.
///
/// Enable `layer-otel-otlp-file` and/or `layer-otel-otlp` features to include stubs for those
/// layers. These must match the features enabled on `pyo3-tracing-subscriber` in your regular
/// dependencies.
///
/// # Errors
///
/// Will return an error if the stub files cannot be written to the given directory.
pub fn write_stub_files(
    host_package: &str,
    tracing_subscriber_module_name: &str,
    directory: &Path,
) -> Result<(), Error> {
    let mut hb = handlebars::Handlebars::new();
    include_stub_and_init!(directory, "", hb);
    hb.register_template_string(
        ".stubtest-allowlist",
        include_str!("../assets/python_stubs/.stubtest-allowlist"),
    )
    .map_err(Box::new)
    .map_err(Error::from)?;
    include_stub_and_init!(directory, "common/", hb);
    include_stub_and_init!(directory, "subscriber/", hb);
    include_stub_and_init!(directory, "layers/", hb);
    include_stub_and_init!(directory, "layers/file/", hb);
    #[cfg(feature = "layer-otel-otlp-file")]
    include_stub_and_init!(directory, "layers/otel_otlp_file/", hb);
    #[cfg(feature = "layer-otel-otlp")]
    include_stub_and_init!(directory, "layers/otel_otlp/", hb);
    let data = Data::new(
        host_package.to_string(),
        tracing_subscriber_module_name.to_string(),
    );
    for name in hb.get_templates().keys() {
        let writer = std::fs::File::create(directory.join(name)).map_err(Error::from)?;
        hb.render_to_write(name, &data, writer)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    #[rstest]
    fn test_build_stub_files() {
        super::write_stub_files(
            "example",
            "_tracing_subscriber",
            std::path::Path::new("target/stubs"),
        )
        .unwrap();
    }
}

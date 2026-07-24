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

//! This module contains a limited set of tracing layers which can be used to configure the
//! [`tracing_subscriber::Registry`] for use with the [`crate::Tracing`] context manager.
//!
//! Currently, the following layers are supported:
//!
//! * [`crate::layers::fmt_file::Config`] - a layer which writes spans to a file (or stdout) in
//! * [`crate::layers::otel_otlp_file::Config`] - a layer which writes spans to a file (or stdout) in
//!   the `OpenTelemetry` OTLP JSON-serialized format.
//! * [`crate::layers::otel_otlp::Config`] - a layer which exports spans to an `OpenTelemetry` collector.
pub(crate) mod fmt_file;
#[cfg(feature = "layer-otel-otlp")]
pub(crate) mod otel_otlp;
#[cfg(feature = "layer-otel-otlp-file")]
pub(crate) mod otel_otlp_file;

use qcs_dependencies_client::opentelemetry_sdk;

use std::fmt::Debug;

use pyo3::prelude::*;
use tracing_subscriber::{
    filter::{FromEnvError, ParseError},
    EnvFilter, Layer, Registry,
};

pub(super) type Shutdown = Box<
    dyn (FnOnce() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = ShutdownResult<()>> + Send + Sync>,
        >) + Send
        + Sync,
>;

/// Carries the built tracing subscriber layer and a shutdown function that can later be used to
/// shutdown the subscriber upon context manager exit.
pub(crate) struct WithShutdown {
    pub(crate) layer: Box<dyn Layer<Registry> + Send + Sync>,
    pub(crate) shutdown: Shutdown,
}

impl Debug for WithShutdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LayerWithShutdown {{ layer: Box<dyn Layer<Registry> + Send + Sync>, shutdown: Shutdown }}")
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum BuildError {
    #[cfg(feature = "layer-otel-otlp-file")]
    #[error("file layer: {0}")]
    File(#[from] otel_otlp_file::BuildError),
    #[cfg(feature = "layer-otel-otlp")]
    #[error("otlp layer: {0}")]
    Otlp(#[from] otel_otlp::BuildError),
    #[error("fmt layer: {0}")]
    FmtFile(#[from] fmt_file::BuildError),
    #[error("failed to parse specified trace filter: {0}")]
    TraceFilterParseError(#[from] ParseError),
    #[error("failed to parse trace filter from RUST_LOG: {0}")]
    TraceFilterEnvError(#[from] FromEnvError),
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub(crate) enum ShutdownError {
    // This will eventually accept a `CustomError` that can be set by upstream libraries.
    // See https://github.com/rigetti/pyo3-opentelemetry/issues/4
    #[error("subscriber shutdown failed: {0}")]
    OTel(#[from] opentelemetry_sdk::error::OTelSdkError),
}

pub(crate) type ShutdownResult<T> = Result<T, ShutdownError>;

pub(super) type LayerBuildResult<T> = Result<T, BuildError>;

pub(crate) trait Config: Send + Sync + BoxDynConfigClone + Debug {
    fn build(&self, batch: bool) -> LayerBuildResult<WithShutdown>;
    fn requires_runtime(&self) -> bool;
}

pub(crate) trait BoxDynConfigClone {
    fn clone_box(&self) -> Box<dyn Config>;
}

/// This trait is necessary so that `Box<dyn Config>` can be cloned and, therefore,
/// used as an attribute on a `pyo3` class.
impl<T> BoxDynConfigClone for T
where
    T: 'static + Config + Clone,
{
    fn clone_box(&self) -> Box<dyn Config> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Config> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[cfg(any(feature = "layer-otel-otlp", feature = "layer-otel-otlp-file"))]
pub(super) fn force_flush_provider_as_shutdown(
    provider: opentelemetry_sdk::trace::SdkTracerProvider,
    timeout: Option<std::time::Duration>,
) -> Shutdown {
    Box::new(
        move || -> std::pin::Pin<Box<dyn std::future::Future<Output = ShutdownResult<()>> + Send + Sync>> {
            Box::pin(async move {
                // TODO: Should this be forwarded to the provider instead? How is this a timeout?
                if let Some(timeout) = timeout {
                    tokio::time::sleep(timeout).await;
                }
                provider.force_flush()?;
                provider.shutdown()?;
                Ok(())
            })
        },
    )
}

/// An environment variable that can be used to set an [`EnvFilter`] for the OTLP layer.
/// This supersedes the `RUST_LOG` environment variable, but is superseded by an explicit
/// `env_filter` argument specified on any layer configuration.
const PYO3_OPENTELEMETRY_ENV_FILTER: &str = "PYO3_OPENTELEMETRY_ENV_FILTER";

pub(super) fn build_env_filter(env_filter: Option<String>) -> Result<EnvFilter, BuildError> {
    env_filter
        .or_else(|| std::env::var(PYO3_OPENTELEMETRY_ENV_FILTER).ok())
        .or_else(|| std::env::var(EnvFilter::DEFAULT_ENV).ok())
        .map_or_else(
            || Ok(EnvFilter::from_default_env()),
            |filter| EnvFilter::try_new(filter).map_err(BuildError::from),
        )
}

/// A Python union of one of the supported layers.
#[derive(FromPyObject, Clone, Debug)]
#[allow(variant_size_differences, clippy::large_enum_variant)]
pub(crate) enum PyConfig {
    #[cfg(feature = "layer-otel-otlp-file")]
    OtlpFile(otel_otlp_file::Config),
    #[cfg(feature = "layer-otel-otlp")]
    Otlp(otel_otlp::PyConfig),
    File(fmt_file::Config),
}

impl Default for PyConfig {
    fn default() -> Self {
        Self::File(fmt_file::Config::default())
    }
}

impl Config for PyConfig {
    fn build(&self, batch: bool) -> LayerBuildResult<WithShutdown> {
        match self {
            #[cfg(feature = "layer-otel-otlp-file")]
            Self::OtlpFile(config) => config.build(batch),
            #[cfg(feature = "layer-otel-otlp")]
            Self::Otlp(config) => config.build(batch),
            Self::File(config) => config.build(batch),
        }
    }

    fn requires_runtime(&self) -> bool {
        match self {
            #[cfg(feature = "layer-otel-otlp-file")]
            Self::OtlpFile(config) => config.requires_runtime(),
            #[cfg(feature = "layer-otel-otlp")]
            Self::Otlp(config) => config.requires_runtime(),
            Self::File(config) => config.requires_runtime(),
        }
    }
}

/// Adds `layers` submodule to the root level submodule.
#[allow(dead_code)]
pub(crate) fn init_submodule<'py>(
    name: &str,
    py: Python<'py>,
    m: &Bound<'py, PyModule>,
) -> PyResult<()> {
    let modules = py.import("sys")?.getattr("modules")?;

    #[cfg(feature = "layer-otel-otlp-file")]
    {
        let submod = pyo3::types::PyModule::new(py, "otel_otlp_file")?;
        let qualified_name = format!("{name}.otel_otlp_file");
        otel_otlp_file::init_submodule(qualified_name.as_str(), py, &submod)?;
        m.add_submodule(&submod)?;
        modules.set_item(qualified_name, submod)?;
    }
    #[cfg(feature = "layer-otel-otlp")]
    {
        let submod = pyo3::types::PyModule::new(py, "otel_otlp")?;
        let qualified_name = format!("{name}.otel_otlp");
        otel_otlp::init_submodule(qualified_name.as_str(), py, &submod)?;
        m.add_submodule(&submod)?;
        modules.set_item(qualified_name, submod)?;
    }

    let submod = pyo3::types::PyModule::new(py, "file")?;
    let qualified_name = format!("{name}.file");
    fmt_file::init_submodule(qualified_name.as_str(), py, &submod)?;
    m.add_submodule(&submod)?;
    modules.set_item(qualified_name, submod)?;

    Ok(())
}

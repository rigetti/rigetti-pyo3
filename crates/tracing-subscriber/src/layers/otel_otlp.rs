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

use std::{collections::HashMap, time::Duration};

use pyo3::prelude::*;
use qcs_dependencies_client::opentelemetry::{trace::TracerProvider, InstrumentationScope};
use qcs_dependencies_client::opentelemetry_otlp::{self, WithExportConfig, WithTonicConfig};
use qcs_dependencies_client::opentelemetry_sdk::{
    self,
    trace::{Sampler, SpanLimits},
    Resource,
};
use qcs_dependencies_client::tonic::metadata::{
    errors::{InvalidMetadataKey, InvalidMetadataValue},
    MetadataKey,
};
use tracing_subscriber::Layer;

use crate::create_init_submodule;
use tracing_subscriber::filter::{FromEnvError, ParseError};

use super::{build_env_filter, force_flush_provider_as_shutdown, LayerBuildResult, WithShutdown};
use crate::common::PyInstrumentationLibrary;

/// Configures the [`opentelemetry_otlp`] crate layer.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    /// Configuration to limit the amount of trace data collected.
    span_limits: SpanLimits,
    /// OpenTelemetry resource attributes describing the entity that produced the telemetry.
    resource: Resource,
    /// The metadata map to use for requests to the remote collector.
    metadata_map: Option<qcs_dependencies_client::tonic::metadata::MetadataMap>,
    /// The sampler to use for the [`opentelemetry_sdk::trace::SdkTracerProvider`].
    sampler: Sampler,
    /// The endpoint to which the exporter will send trace data. If not set, this must be set by
    /// OTLP environment variables.
    endpoint: Option<String>,
    /// Timeout applied the [`qcs_dependencies_client::tonic::transport::Channel`] used to send trace data to the remote collector.
    timeout: Option<Duration>,
    /// A timeout applied to the shutdown of the [`crate::contextmanager::Tracing`] context
    /// manager upon exiting, before the underlying [`opentelemetry_sdk::trace::SdkTracerProvider`]
    /// is shutdown. Ensures that spans are flushed before the program exits.
    pre_shutdown_timeout: Duration,
    /// The filter to use for the [`tracing_subscriber::filter::EnvFilter`] layer.
    filter: Option<String>,
    /// The instrumentation library to use for the [`opentelemetry_sdk::trace::SdkTracerProvider`].
    instrumentation_library: Option<InstrumentationScope>,
}

impl Config {
    fn initialize_otlp_exporter(&self) -> LayerBuildResult<opentelemetry_otlp::SpanExporter> {
        let mut builder = opentelemetry_otlp::SpanExporter::builder().with_tonic();
        if let Some(endpoint) = self.endpoint.clone() {
            builder = builder.with_endpoint(endpoint);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout(timeout);
        }
        if let Some(metadata_map) = self.metadata_map.clone() {
            builder = builder.with_metadata(metadata_map);
        }
        let otlp_exporter = builder.build().map_err(BuildError::from)?;
        Ok(otlp_exporter)
    }
}

impl crate::layers::Config for PyConfig {
    fn requires_runtime(&self) -> bool {
        Config::requires_runtime()
    }
    fn build(&self, batch: bool) -> LayerBuildResult<WithShutdown> {
        Config::try_from(self.clone())?.build(batch)
    }
}

impl Config {
    const fn requires_runtime() -> bool {
        true
    }

    fn build(&self, batch: bool) -> LayerBuildResult<WithShutdown> {
        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_sampler(self.sampler.clone())
            .with_span_limits(self.span_limits)
            .with_resource(self.resource.clone());

        let exporter = self.initialize_otlp_exporter()?;
        let provider = if batch {
            provider.with_batch_exporter(exporter)
        } else {
            provider.with_simple_exporter(exporter)
        }
        .build();
        let env_filter = build_env_filter(self.filter.clone())?;

        let tracer = self.instrumentation_library.as_ref().map_or_else(
            || provider.tracer("pyo3_tracing_subscriber"),
            |instrumentation_library| provider.tracer_with_scope(instrumentation_library.clone()),
        );

        let layer = qcs_dependencies_client::tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(env_filter);
        Ok(WithShutdown {
            layer: Box::new(layer),
            shutdown: force_flush_provider_as_shutdown(provider, Some(self.pre_shutdown_timeout)),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum BuildError {
    #[error("failed to build opentelemetry-otlp pipeline: {0}")]
    TraceInstall(#[from] opentelemetry_otlp::ExporterBuildError),
    #[error("error in the configuration: {0}")]
    Config(#[from] ConfigError),
    #[error("failed to parse specified trace filter: {0}")]
    TraceFilterParseError(#[from] ParseError),
    #[error("failed to parse trace filter from RUST_LOG: {0}")]
    TraceFilterEnvError(#[from] FromEnvError),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigError {
    #[error("invalid metadata map value: {0}")]
    InvalidMetadataValue(#[from] InvalidMetadataValue),
    #[error("invalid metadata map key: {0}")]
    InvalidMetadataKey(#[from] InvalidMetadataKey),
}

#[pyclass(name = "SpanLimits")]
#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
struct PySpanLimits {
    /// The max events that can be added to a `Span`.
    max_events_per_span: u32,
    /// The max attributes that can be added to a `Span`.
    max_attributes_per_span: u32,
    /// The max links that can be added to a `Span`.
    max_links_per_span: u32,
    /// The max attributes that can be added into an `Event`
    max_attributes_per_event: u32,
    /// The max attributes that can be added into a `Link`
    max_attributes_per_link: u32,
}

impl Default for PySpanLimits {
    fn default() -> Self {
        Self::from(SpanLimits::default())
    }
}

impl From<SpanLimits> for PySpanLimits {
    fn from(span_limits: SpanLimits) -> Self {
        Self {
            max_events_per_span: span_limits.max_events_per_span,
            max_attributes_per_span: span_limits.max_attributes_per_span,
            max_links_per_span: span_limits.max_links_per_span,
            max_attributes_per_event: span_limits.max_attributes_per_event,
            max_attributes_per_link: span_limits.max_attributes_per_link,
        }
    }
}

impl From<PySpanLimits> for SpanLimits {
    fn from(span_limits: PySpanLimits) -> Self {
        Self {
            max_events_per_span: span_limits.max_events_per_span,
            max_attributes_per_span: span_limits.max_attributes_per_span,
            max_links_per_span: span_limits.max_links_per_span,
            max_attributes_per_event: span_limits.max_attributes_per_event,
            max_attributes_per_link: span_limits.max_attributes_per_link,
        }
    }
}

#[pymethods]
impl PySpanLimits {
    #[new]
    #[pyo3(signature = (
        /,
        max_events_per_span = None,
        max_attributes_per_span = None,
        max_links_per_span = None,
        max_attributes_per_event = None,
        max_attributes_per_link = None
    ))]
    fn new(
        max_events_per_span: Option<u32>,
        max_attributes_per_span: Option<u32>,
        max_links_per_span: Option<u32>,
        max_attributes_per_event: Option<u32>,
        max_attributes_per_link: Option<u32>,
    ) -> Self {
        let span_limits = Self::default();
        Self {
            max_events_per_span: max_events_per_span.unwrap_or(span_limits.max_events_per_span),
            max_attributes_per_span: max_attributes_per_span
                .unwrap_or(span_limits.max_attributes_per_span),
            max_links_per_span: max_links_per_span.unwrap_or(span_limits.max_links_per_span),
            max_attributes_per_event: max_attributes_per_event
                .unwrap_or(span_limits.max_attributes_per_event),
            max_attributes_per_link: max_attributes_per_link
                .unwrap_or(span_limits.max_attributes_per_link),
        }
    }
}

/// A Python representation of [`Config`].
#[pyclass(name = "Config")]
#[derive(Clone, Default, Debug)]
pub(crate) struct PyConfig {
    span_limits: PySpanLimits,
    resource: PyResource,
    metadata_map: Option<HashMap<String, String>>,
    sampler: PySampler,
    endpoint: Option<String>,
    timeout_millis: Option<u64>,
    pre_shutdown_timeout_millis: u64,
    filter: Option<String>,
    instrumentation_library: Option<PyInstrumentationLibrary>,
}

#[pymethods]
impl PyConfig {
    #[new]
    #[pyo3(signature = (
        /,
        span_limits = None,
        resource = None,
        metadata_map = None,
        sampler = None,
        endpoint = None,
        timeout_millis = None,
        pre_shutdown_timeout_millis = 2000,
        filter = None,
        instrumentation_library = None
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new<'py>(
        span_limits: Option<PySpanLimits>,
        resource: Option<PyResource>,
        metadata_map: Option<&Bound<'py, PyAny>>,
        sampler: Option<&Bound<'py, PyAny>>,
        endpoint: Option<&str>,
        timeout_millis: Option<u64>,
        pre_shutdown_timeout_millis: u64,
        filter: Option<&str>,
        instrumentation_library: Option<PyInstrumentationLibrary>,
    ) -> PyResult<Self> {
        Ok(Self {
            span_limits: span_limits.unwrap_or_default(),
            resource: resource.unwrap_or_default(),
            metadata_map: metadata_map
                .map(pyo3::types::PyAnyMethods::extract)
                .transpose()?,
            sampler: sampler
                .map(pyo3::types::PyAnyMethods::extract)
                .transpose()?
                .unwrap_or_default(),
            endpoint: endpoint.map(String::from),
            timeout_millis,
            pre_shutdown_timeout_millis,
            filter: filter.map(String::from),
            instrumentation_library,
        })
    }
}

#[pyclass(name = "Resource")]
#[derive(Clone, Default, Debug)]
struct PyResource {
    attrs: HashMap<String, PyResourceValue>,
    schema_url: Option<String>,
}

#[pymethods]
impl PyResource {
    #[new]
    #[pyo3(signature = (/, attrs = None, schema_url = None))]
    fn new(attrs: Option<HashMap<String, PyResourceValue>>, schema_url: Option<&str>) -> Self {
        Self {
            attrs: attrs.unwrap_or_default(),
            schema_url: schema_url.map(String::from),
        }
    }
}

impl From<PyResource> for Resource {
    fn from(resource: PyResource) -> Self {
        let kvs = resource
            .attrs
            .into_iter()
            .map(|(k, v)| qcs_dependencies_client::opentelemetry::KeyValue::new(k, v));

        if let Some(schema_url) = resource.schema_url {
            Self::builder().with_schema_url(kvs, schema_url)
        } else {
            Self::builder().with_attributes(kvs)
        }
        .build()
    }
}

#[derive(FromPyObject, Clone, Debug, PartialEq)]
pub(crate) enum PyResourceValue {
    /// bool values
    Bool(bool),
    /// i64 values
    I64(i64),
    /// f64 values
    F64(f64),
    /// String values
    String(String),
    /// Array of homogeneous values
    Array(PyResourceValueArray),
}

#[derive(FromPyObject, Debug, Clone, PartialEq)]
pub(crate) enum PyResourceValueArray {
    /// Array of bools
    Bool(Vec<bool>),
    /// Array of integers
    I64(Vec<i64>),
    /// Array of floats
    F64(Vec<f64>),
    /// Array of strings
    String(Vec<String>),
}

impl From<PyResourceValueArray> for qcs_dependencies_client::opentelemetry::Array {
    fn from(py_resource_value_array: PyResourceValueArray) -> Self {
        match py_resource_value_array {
            PyResourceValueArray::Bool(b) => Self::Bool(b),
            PyResourceValueArray::I64(i) => Self::I64(i),
            PyResourceValueArray::F64(f) => Self::F64(f),
            PyResourceValueArray::String(s) => {
                Self::String(s.iter().map(|v| v.clone().into()).collect())
            }
        }
    }
}

impl From<PyResourceValue> for qcs_dependencies_client::opentelemetry::Value {
    fn from(py_resource_value: PyResourceValue) -> Self {
        match py_resource_value {
            PyResourceValue::Bool(b) => Self::Bool(b),
            PyResourceValue::I64(i) => Self::I64(i),
            PyResourceValue::F64(f) => Self::F64(f),
            PyResourceValue::String(s) => Self::String(s.into()),
            PyResourceValue::Array(a) => Self::Array(a.into()),
        }
    }
}

#[allow(variant_size_differences)]
#[derive(FromPyObject, Debug, Clone, PartialEq)]
enum PySampler {
    AlwaysOn(bool),
    TraceIdParentRatioBased(f64),
}

impl Default for PySampler {
    fn default() -> Self {
        Self::AlwaysOn(true)
    }
}

impl From<PySampler> for Sampler {
    fn from(sampler: PySampler) -> Self {
        match sampler {
            PySampler::AlwaysOn(b) if b => Self::AlwaysOn,
            PySampler::AlwaysOn(_) => Self::AlwaysOff,
            PySampler::TraceIdParentRatioBased(f) => Self::TraceIdRatioBased(f),
        }
    }
}

/// The Rust `OpenTelemetry` SDK does not support the official OTLP headers [environment variables](https://opentelemetry.io/docs/specs/otel/protocol/exporter/).
/// Here we include a custom implementation.
const OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";
const OTEL_EXPORTER_OTLP_TRACES_HEADERS: &str = "OTEL_EXPORTER_OTLP_TRACES_HEADERS";

fn get_metadata_from_environment(
) -> Result<qcs_dependencies_client::tonic::metadata::MetadataMap, ConfigError> {
    [
        OTEL_EXPORTER_OTLP_HEADERS,
        OTEL_EXPORTER_OTLP_TRACES_HEADERS,
    ]
    .iter()
    .filter_map(|k| std::env::var(k).ok())
    .flat_map(|headers| {
        headers
            .split(',')
            .map(String::from)
            .filter_map(|kv| {
                let mut x = kv.split('=').map(String::from);
                Some((x.next()?, x.next()?))
            })
            .collect::<Vec<(String, String)>>()
    })
    .try_fold(
        qcs_dependencies_client::tonic::metadata::MetadataMap::new(),
        |mut metadata, (k, v)| {
            let key = k.parse::<MetadataKey<_>>().map_err(ConfigError::from)?;
            metadata.insert(key, v.parse().map_err(ConfigError::from)?);
            Ok(metadata)
        },
    )
}

impl TryFrom<PyConfig> for Config {
    type Error = BuildError;

    fn try_from(config: PyConfig) -> Result<Self, Self::Error> {
        let env_metadata_map = get_metadata_from_environment()?;
        let metadata_map = match config.metadata_map {
            Some(m) => Some(m.into_iter().try_fold(
                env_metadata_map,
                |mut metadata_map: qcs_dependencies_client::tonic::metadata::MetadataMap,
                 (k, v)|
                 -> Result<_, Self::Error> {
                    let key = k.parse::<MetadataKey<_>>().map_err(ConfigError::from)?;
                    metadata_map.insert(key, v.parse().map_err(ConfigError::from)?);
                    Ok(metadata_map)
                },
            )?),
            None if !env_metadata_map.is_empty() => Some(env_metadata_map),
            None => None,
        };

        Ok(Self {
            span_limits: config.span_limits.into(),
            resource: config.resource.into(),
            metadata_map,
            sampler: config.sampler.into(),
            endpoint: config.endpoint,
            timeout: config.timeout_millis.map(Duration::from_millis),
            pre_shutdown_timeout: Duration::from_millis(config.pre_shutdown_timeout_millis),
            filter: config.filter,
            instrumentation_library: config.instrumentation_library.map(Into::into),
        })
    }
}

create_init_submodule! {
    classes: [ PyConfig, PySpanLimits, PyResource ],
}

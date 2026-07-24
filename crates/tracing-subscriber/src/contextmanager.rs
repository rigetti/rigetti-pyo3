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
use pyo3::{prelude::*, types::PyNone};
use rigetti_pyo3::{exception, sync::Awaitable};

use super::export_process::{ExportProcess, ExportProcessConfig};

#[pyclass]
#[derive(Clone, Debug, Default)]
pub(crate) struct GlobalTracingConfig {
    pub(crate) export_process: ExportProcessConfig,
}

#[pymethods]
impl GlobalTracingConfig {
    #[new]
    #[pyo3(signature = (/, export_process = None))]
    #[allow(clippy::pedantic)]
    fn new(export_process: Option<ExportProcessConfig>) -> PyResult<Self> {
        let export_process = export_process.unwrap_or_default();
        Ok(Self { export_process })
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub(crate) struct CurrentThreadTracingConfig {
    pub(crate) export_process: ExportProcessConfig,
}

#[pymethods]
impl CurrentThreadTracingConfig {
    #[new]
    #[pyo3(signature = (/, export_process = None))]
    #[allow(clippy::pedantic)]
    fn new(export_process: Option<ExportProcessConfig>) -> PyResult<Self> {
        let export_process = export_process.unwrap_or_default();
        Ok(Self { export_process })
    }
}

#[derive(FromPyObject, Debug)]
pub(crate) enum TracingConfig {
    Global(GlobalTracingConfig),
    CurrentThread(CurrentThreadTracingConfig),
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self::Global(GlobalTracingConfig::default())
    }
}

/// Represents the current state of the context manager. This state is used to ensure the
/// context manager methods are invoked in the correct order and multiplicity.
#[derive(Debug)]
enum ContextManagerState {
    Initialized(TracingConfig),
    Entered(ExportProcess),
    Starting,
    Exited,
}

/// A Python class that implements the context manager interface. It is initialized with a
/// configuration. Upon entry it builds and installs the configured tracing subscriber. Upon exit
/// it shuts down the tracing subscriber.
#[pyclass]
#[derive(Debug)]
pub struct Tracing {
    state: ContextManagerState,
}

#[derive(thiserror::Error, Debug)]
enum ContextManagerError {
    #[error("entered tracing context manager with no configuration defined; ensure contextmanager only enters once")]
    EnterWithoutConfiguration,
    #[error("exited tracing context manager with no export process defined; ensure contextmanager only exits once after being entered")]
    ExitWithoutExportProcess,
}

exception!(
    ContextManagerError,
    contextmanager,
    TracingContextManagerError,
    pyo3::exceptions::PyRuntimeError,
    "Errors generated through use of the tracing context manager."
);

#[pymethods]
impl Tracing {
    #[new]
    #[pyo3(signature = (/, config = None))]
    #[allow(clippy::pedantic)]
    fn new(config: Option<TracingConfig>) -> PyResult<Self> {
        let config = config.unwrap_or_default();
        Ok(Self {
            state: ContextManagerState::Initialized(config),
        })
    }

    fn __enter__(&mut self) -> PyResult<()> {
        let state = std::mem::replace(&mut self.state, ContextManagerState::Starting);
        if let ContextManagerState::Initialized(config) = state {
            self.state = ContextManagerState::Entered(ExportProcess::start(config)?);
        } else {
            Err(ContextManagerError::EnterWithoutConfiguration)?;
        }
        Ok(())
    }

    fn __aenter__<'py>(&mut self, py: Python<'py>) -> PyResult<Awaitable<'py, PyNone>> {
        self.__enter__()?;
        pyo3_async_runtimes::tokio::future_into_py(py, async { Ok(()) }).map(Into::into)
    }

    fn __exit__<'py>(
        &mut self,
        _exc_type: Option<Bound<'py, PyAny>>,
        _exc_value: Option<Bound<'py, PyAny>>,
        _traceback: Option<Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        let state = std::mem::replace(&mut self.state, ContextManagerState::Exited);
        if let ContextManagerState::Entered(export_process) = state {
            let py_rt = pyo3_async_runtimes::tokio::get_runtime();
            // Why block and not run this in a future within aexit? The `shutdown`
            // method returns a Tokio runtime, which cannot be dropped within another
            // runtime. Additionally, `pyo3_async_runtimes::tokio::future_into_py` futures
            // must resolve to something that implements `IntoPyObject`.
            if let Some(export_runtime) = py_rt.block_on(export_process.shutdown())? {
                // This immediately shuts the runtime down. The expectation here is that the
                // process shutdown is responsible for cleaning up all background tasks and
                // shutting down gracefully.
                export_runtime.shutdown_background();
            }
        } else {
            Err(ContextManagerError::ExitWithoutExportProcess)?;
        }

        Ok(())
    }

    fn __aexit__<'py>(
        &mut self,
        py: Python<'py>,
        exc_type: Option<Bound<'py, PyAny>>,
        exc_value: Option<Bound<'py, PyAny>>,
        traceback: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Awaitable<'py, PyNone>> {
        self.__exit__(exc_type, exc_value, traceback)?;
        pyo3_async_runtimes::tokio::future_into_py(py, async { Ok(()) }).map(Into::into)
    }
}

#[cfg(feature = "layer-otel-otlp-file")]
#[cfg(test)]
mod test {
    use std::{
        env::temp_dir,
        io::BufRead,
        path::PathBuf,
        thread::sleep,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use tokio::runtime::Builder;

    use crate::{
        contextmanager::{CurrentThreadTracingConfig, GlobalTracingConfig, TracingConfig},
        export_process::{ExportProcess, ExportProcessConfig, SimpleConfig},
        subscriber::TracingSubscriberRegistryConfig,
    };
    use qcs_dependencies_client::opentelemetry_proto::tonic::trace::v1 as otlp;

    #[tracing::instrument]
    fn example() {
        sleep(SPAN_DURATION);
    }

    const N_SPANS: usize = 5;
    const SPAN_DURATION: Duration = Duration::from_millis(100);

    #[test]
    /// Test that a global simple export process can be started and stopped and that it
    /// exports accurate spans as configured.
    fn test_global_simple() {
        let temporary_file_path = get_tempfile("test_global_simple");
        let layer_config = Box::new(crate::layers::otel_otlp_file::Config {
            file_path: Some(temporary_file_path.as_os_str().to_str().unwrap().to_owned()),
            filter: Some("error,pyo3_tracing_subscriber=info".to_string()),
            instrumentation_library: None,
        });
        let subscriber = Box::new(TracingSubscriberRegistryConfig { layer_config });
        let config = TracingConfig::Global(GlobalTracingConfig {
            export_process: ExportProcessConfig::Simple(SimpleConfig {
                subscriber: crate::subscriber::PyConfig {
                    subscriber_config: subscriber,
                },
            }),
        });
        let export_process = ExportProcess::start(config).unwrap();
        let rt2 = Builder::new_current_thread().enable_time().build().unwrap();
        let _guard = rt2.enter();
        let runtime = rt2
            .block_on(tokio::time::timeout(Duration::from_secs(1), async move {
                for _ in 0..N_SPANS {
                    example();
                }
                export_process.shutdown().await
            }))
            .unwrap()
            .unwrap();
        assert!(runtime.is_none());

        let reader = std::io::BufReader::new(std::fs::File::open(temporary_file_path).unwrap());
        let lines = reader.lines();
        let spans = lines
            .flat_map(|line| {
                let line = line.unwrap();
                let span_data: otlp::TracesData =
                    serde_json::from_str(line.as_str().trim()).unwrap();
                span_data
                    .resource_spans
                    .iter()
                    .flat_map(|resource_span| {
                        resource_span
                            .scope_spans
                            .iter()
                            .flat_map(|scope_span| scope_span.spans.clone())
                    })
                    .collect::<Vec<otlp::Span>>()
            })
            .collect::<Vec<otlp::Span>>();
        assert_eq!(spans.len(), N_SPANS);

        let span_grace = Duration::from_millis(50);
        for span in spans {
            assert_eq!(span.name, "example");
            assert!(
                u128::from(span.end_time_unix_nano - span.start_time_unix_nano)
                    >= SPAN_DURATION.as_nanos()
            );
            assert!(
                u128::from(span.end_time_unix_nano - span.start_time_unix_nano)
                    <= (SPAN_DURATION.as_nanos() + span_grace.as_nanos())
            );
        }
    }

    fn get_tempfile(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be able to get current time")
            .as_nanos();
        let dir = temp_dir();
        dir.join(std::path::Path::new(
            format!("{prefix}-{timestamp}.txt").as_str(),
        ))
    }

    #[test]
    /// Test that a current thread simple export process can be started and stopped and that it
    /// exports accurate spans as configured.
    fn test_current_thread_simple() {
        let temporary_file_path = get_tempfile("test_current_thread_simple");
        let layer_config = Box::new(crate::layers::otel_otlp_file::Config {
            file_path: Some(temporary_file_path.as_os_str().to_str().unwrap().to_owned()),
            filter: Some("error,pyo3_tracing_subscriber=info".to_string()),
            instrumentation_library: None,
        });
        let subscriber = Box::new(TracingSubscriberRegistryConfig { layer_config });
        let config = TracingConfig::CurrentThread(CurrentThreadTracingConfig {
            export_process: crate::export_process::ExportProcessConfig::Simple(SimpleConfig {
                subscriber: crate::subscriber::PyConfig {
                    subscriber_config: subscriber,
                },
            }),
        });
        let export_process = ExportProcess::start(config).unwrap();

        for _ in 0..N_SPANS {
            example();
        }

        let rt2 = Builder::new_current_thread().enable_time().build().unwrap();
        let _guard = rt2.enter();
        let runtime = rt2
            .block_on(tokio::time::timeout(Duration::from_secs(1), async move {
                export_process.shutdown().await
            }))
            .unwrap()
            .unwrap();
        assert!(runtime.is_none());

        let reader = std::io::BufReader::new(std::fs::File::open(temporary_file_path).unwrap());
        let lines = reader.lines();
        let spans = lines
            .flat_map(|line| {
                let line = line.unwrap();
                let span_data: otlp::TracesData = serde_json::from_str(line.as_str()).unwrap();
                span_data
                    .resource_spans
                    .iter()
                    .flat_map(|resource_span| {
                        resource_span
                            .scope_spans
                            .iter()
                            .flat_map(|scope_span| scope_span.spans.clone())
                    })
                    .collect::<Vec<otlp::Span>>()
            })
            .collect::<Vec<otlp::Span>>();
        assert_eq!(spans.len(), N_SPANS);

        let span_grace = Duration::from_millis(50);
        for span in spans {
            assert_eq!(span.name, "example");
            assert!(
                u128::from(span.end_time_unix_nano - span.start_time_unix_nano)
                    >= SPAN_DURATION.as_nanos()
            );
            assert!(
                u128::from(span.end_time_unix_nano - span.start_time_unix_nano)
                    <= (SPAN_DURATION.as_nanos() + span_grace.as_nanos())
            );
        }
    }
}

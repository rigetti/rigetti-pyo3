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

use std::fmt::Debug;

use crate::subscriber::PyConfig;
use pyo3::prelude::*;
use rigetti_pyo3::exception;
use tokio::runtime::Runtime;

use super::{
    contextmanager::TracingConfig,
    subscriber::{self, set_subscriber, SetSubscriberError, SubscriberManagerGuard},
};

mod background;

/// Configuration for batch export processes. Batch export processes typically accumulate
/// trace data in memory and export that data in batch. This is favorable in
/// most situations to reduce the amount of I/O required to export trace data. See
/// `opentelemetry_sdk::trace::BatchSpanProcessor` for more details.
#[pyclass]
#[derive(Clone, Debug, Default)]
pub(crate) struct BatchConfig {
    pub(super) subscriber: PyConfig,
}

#[pymethods]
impl BatchConfig {
    #[new]
    #[pyo3(signature = (subscriber = None))]
    #[allow(clippy::pedantic)]
    fn new(subscriber: Option<PyConfig>) -> PyResult<Self> {
        let subscriber = subscriber.unwrap_or_default();
        Ok(Self { subscriber })
    }
}

/// Configuration for simple export processes. A simple export process does not accumulate
/// trace data in memory, but instead exports each trace event as it is received. This may
/// be favorable in situations where the amount of trace data is expected to be small and
/// the overhead of background processing is not worth it. See
/// `opentelemetry_sdk::trace::SimpleSpanProcessor` for more details.
#[pyclass]
#[derive(Clone, Debug, Default)]
pub(crate) struct SimpleConfig {
    pub(super) subscriber: PyConfig,
}

#[pymethods]
impl SimpleConfig {
    #[new]
    #[pyo3(signature = (subscriber = None))]
    #[allow(clippy::pedantic)]
    fn new(subscriber: Option<PyConfig>) -> PyResult<Self> {
        let subscriber = subscriber.unwrap_or_default();
        Ok(Self { subscriber })
    }
}

#[derive(FromPyObject, Clone, Debug)]
pub(crate) enum ExportProcessConfig {
    Batch(BatchConfig),
    Simple(SimpleConfig),
}

impl Default for ExportProcessConfig {
    fn default() -> Self {
        Self::Batch(BatchConfig::default())
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum StartError {
    #[error("failed to start global batch: {0}")]
    GlobalBatch(#[from] background::StartError),
    #[error("failed to build subscriber {0}")]
    BuildSubscriber(#[from] subscriber::BuildError),
    #[error("failed to set subscriber: {0}")]
    SetSubscriber(#[from] SetSubscriberError),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ShutdownError {
    #[error("the subscriber failed to shutdown: {0}")]
    Subscriber(#[from] crate::subscriber::ShutdownError),
}

exception!(
    StartError,
    export_process,
    TracingStartError,
    pyo3::exceptions::PyRuntimeError,
    "Errors encounter if starting tracing fails."
);

exception!(
    ShutdownError,
    export_process,
    TracingShutdownError,
    pyo3::exceptions::PyRuntimeError,
    "Errors encounter if shutting down tracing fails."
);

type StartResult<T> = Result<T, StartError>;
type ShutdownResult<T> = Result<T, ShutdownError>;

/// A representation of a running export process, either a background task or a process just
/// running in the current thread. A background task carries both its tokio runtime and the
/// tracing subscriber guard; a foreground task only carries the subscriber guard.
pub(crate) enum ExportProcess {
    Background(background::ExportProcess),
    Foreground(SubscriberManagerGuard),
}

impl Debug for ExportProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Background(_) => f
                .debug_struct("ExportProcess::Background")
                .field("process", &"process")
                .finish(),
            Self::Foreground(_) => f
                .debug_struct("ExportProcess::Foreground")
                .field("guard", &"guard")
                .finish(),
        }
    }
}

impl ExportProcess {
    pub(crate) fn start(config: TracingConfig) -> StartResult<Self> {
        match config {
            TracingConfig::Global(config) => match config.export_process {
                ExportProcessConfig::Batch(config) => Ok(Self::Background(
                    background::ExportProcess::start(config.subscriber.subscriber_config, true)?,
                )),
                ExportProcessConfig::Simple(config) => {
                    let requires_runtime = config.subscriber.subscriber_config.requires_runtime();
                    if requires_runtime {
                        Ok(Self::Background(background::ExportProcess::start(
                            config.subscriber.subscriber_config,
                            true,
                        )?))
                    } else {
                        let subscriber = config.subscriber.subscriber_config.build(false)?;
                        Ok(Self::Foreground(set_subscriber(subscriber, true)?))
                    }
                }
            },
            TracingConfig::CurrentThread(config) => match config.export_process {
                ExportProcessConfig::Batch(config) => Ok(Self::Background(
                    background::ExportProcess::start(config.subscriber.subscriber_config, false)?,
                )),
                ExportProcessConfig::Simple(config) => {
                    let requires_runtime = config.subscriber.subscriber_config.requires_runtime();
                    if requires_runtime {
                        Ok(Self::Background(background::ExportProcess::start(
                            config.subscriber.subscriber_config,
                            false,
                        )?))
                    } else {
                        let subscriber = config.subscriber.subscriber_config.build(false)?;
                        Ok(Self::Foreground(set_subscriber(subscriber, false)?))
                    }
                }
            },
        }
    }

    pub(crate) async fn shutdown(self) -> ShutdownResult<Option<Runtime>> {
        match self {
            Self::Background(process) => Ok(Some(process.shutdown().await?)),
            Self::Foreground(guard) => {
                guard.shutdown().await?;
                Ok(None)
            }
        }
    }
}

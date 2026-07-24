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
use crate::create_init_submodule;
use pyo3::prelude::*;
use tracing_subscriber::Layer;

use super::{build_env_filter, LayerBuildResult, ShutdownResult, WithShutdown};

/// Configures the [`mod@tracing_subscriber::fmt`] layer.
///
/// If `file_path` is None, the layer will write to stdout.
/// This outputs data in a non-OpenTelemetry format.
#[pyclass]
#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) file_path: Option<String>,
    pub(crate) pretty: bool,
    pub(crate) filter: Option<String>,
    pub(crate) json: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file_path: None,
            pretty: false,
            filter: None,
            json: true,
        }
    }
}

#[pymethods]
impl Config {
    #[new]
    #[pyo3(signature = (/, file_path = None, pretty = false, filter = None, json = true))]
    const fn new(
        file_path: Option<String>,
        pretty: bool,
        filter: Option<String>,
        json: bool,
    ) -> Self {
        Self {
            file_path,
            pretty,
            filter,
            json,
        }
    }
}

impl crate::layers::Config for Config {
    fn requires_runtime(&self) -> bool {
        false
    }

    fn build(&self, _batch: bool) -> LayerBuildResult<WithShutdown> {
        let filter = build_env_filter(self.filter.clone())?;
        let layer = if let Some(file_path) = self.file_path.as_ref() {
            let file = std::fs::File::create(file_path).map_err(BuildError::from)?;
            if self.json && self.pretty {
                tracing_subscriber::fmt::layer()
                    .json()
                    .pretty()
                    .with_writer(file)
                    .with_filter(filter)
                    .boxed()
            } else if self.json {
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(file)
                    .with_filter(filter)
                    .boxed()
            } else if self.pretty {
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_writer(file)
                    .with_filter(filter)
                    .boxed()
            } else {
                tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_filter(filter)
                    .boxed()
            }
        } else if self.json && self.pretty {
            tracing_subscriber::fmt::layer()
                .json()
                .pretty()
                .with_filter(filter)
                .boxed()
        } else if self.json {
            tracing_subscriber::fmt::layer()
                .json()
                .with_filter(filter)
                .boxed()
        } else if self.pretty {
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_filter(filter)
                .boxed()
        } else {
            tracing_subscriber::fmt::layer().with_filter(filter).boxed()
        };

        Ok(WithShutdown {
            layer: Box::new(layer),
            shutdown: Box::new(
                move || -> std::pin::Pin<Box<dyn std::future::Future<Output = ShutdownResult<()>> + Send + Sync>> {
                    Box::pin(async move {
                        Ok(())
                    })
                },
            )
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum BuildError {
    #[error("failed to initialize fmt layer for specified file path: {0}")]
    InvalidFile(#[from] std::io::Error),
}

create_init_submodule! {
    classes: [ Config ],
}

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
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::{layer::Layered, prelude::__tracing_subscriber_SubscriberExt, Registry};

#[derive(thiserror::Error, Debug)]
pub(crate) enum ShutdownError {
    #[error("failed to shutdown configured layer: {0}")]
    LayerShutdown(#[from] crate::layers::ShutdownError),
}

type ShutdownResult<T> = Result<T, ShutdownError>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum BuildError {
    #[error("failed to build layer: {0}")]
    LayerBuild(#[from] crate::layers::BuildError),
}

/// A shutdown function that can be used to shutdown the configured tracing subscriber.
pub(crate) type Shutdown = Box<
    dyn (FnOnce() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = ShutdownResult<()>> + Send + Sync>,
        >) + Send
        + Sync,
>;

type SubscriberBuildResult<T> = Result<T, BuildError>;

pub(crate) trait Config: BoxDynConfigClone + Send + Sync {
    /// Indicates whether the underlying layer requires a Tokio runtime when the
    /// layer is built _without_ batch export.
    fn requires_runtime(&self) -> bool;
    /// Builds the configured tracing subscriber. The `batch` argument may be
    /// passed to underlying layers to indicate whether the subscriber will be
    /// used in a batch context.
    fn build(&self, batch: bool) -> SubscriberBuildResult<WithShutdown>;
}

/// This trait is necessary so that `Box<dyn Config>` can be cloned and, therefore,
/// used as an attribute on a `pyo3` class.
pub(crate) trait BoxDynConfigClone {
    fn clone_box(&self) -> Box<dyn Config>;
}

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

/// A built tracing subscriber that is both `Send` and `Sync`. This is necessary to run any
/// background tasks necessary for exporting trace data.
pub(crate) trait SendSyncSubscriber: tracing::subscriber::Subscriber + Send + Sync {}

impl<L, I> SendSyncSubscriber for Layered<L, I>
where
    L: tracing_subscriber::Layer<I> + Send + Sync,
    I: tracing::Subscriber + Send + Sync,
{
}

/// Carries the built tracing subscriber and a shutdown function that can later be used to
/// shutdown the subscriber upon context manager exit.
pub(crate) struct WithShutdown {
    pub(crate) subscriber: Box<dyn SendSyncSubscriber>,
    pub(crate) shutdown: Shutdown,
}

impl core::fmt::Debug for WithShutdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WithShutdown subscriber: Box<dyn SendSyncSubscriber>, shutdown: Shutdown",
        )
    }
}

/// A Python wrapper for a tracing subscriber configuration.
#[pyclass(name = "Config")]
#[derive(Clone)]
pub(crate) struct PyConfig {
    pub(crate) subscriber_config: Box<dyn Config>,
}

impl Default for PyConfig {
    fn default() -> Self {
        let layer = super::layers::PyConfig::default();
        Self {
            subscriber_config: Box::new(TracingSubscriberRegistryConfig {
                layer_config: Box::new(layer),
            }),
        }
    }
}

impl core::fmt::Debug for PyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PyConfig {{ subscriber_config: Box<dyn Config> }}")
    }
}

#[pymethods]
impl PyConfig {
    #[new]
    #[pyo3(signature = (/, layer = None))]
    #[allow(clippy::pedantic)]
    fn new(layer: Option<super::layers::PyConfig>) -> PyResult<Self> {
        let layer = layer.unwrap_or_default();
        Ok(Self {
            subscriber_config: Box::new(TracingSubscriberRegistryConfig {
                layer_config: Box::new(layer),
            }),
        })
    }
}

/// A concrete implementation of [`Config`] that wraps a [`tracing_subscriber::Registry`]. This is
/// used internally to build a [`tracing_subscriber::Registry`] from a [`crate::layers::PyConfig`].
#[derive(Clone)]
pub(super) struct TracingSubscriberRegistryConfig {
    pub(super) layer_config: Box<dyn super::layers::Config>,
}

impl Config for TracingSubscriberRegistryConfig {
    fn requires_runtime(&self) -> bool {
        self.layer_config.requires_runtime()
    }

    fn build(&self, batch: bool) -> SubscriberBuildResult<WithShutdown> {
        let layer = self.layer_config.clone().build(batch)?;
        let subscriber = Registry::default().with(layer.layer);
        let shutdown = layer.shutdown;
        Ok(WithShutdown {
            subscriber: Box::new(subscriber),
            shutdown: Box::new(move || {
                Box::pin(async move {
                    shutdown().await?;
                    Ok(())
                })
            }),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum SetSubscriberError {
    #[error("global default: {0}")]
    SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),
}

type SetSubscriberResult<T> = Result<T, SetSubscriberError>;

/// Sets the tracing subscriber for the current thread or globally. It returns a guard
/// that can be shutdown asynchronously.
pub(crate) fn set_subscriber(
    subscriber: WithShutdown,
    global: bool,
) -> SetSubscriberResult<SubscriberManagerGuard> {
    if global {
        let shutdown = subscriber.shutdown;
        tracing::subscriber::set_global_default(subscriber.subscriber)?;
        Ok(SubscriberManagerGuard::Global(shutdown))
    } else {
        let shutdown = subscriber.shutdown;
        let guard = tracing::subscriber::set_default(subscriber.subscriber);
        Ok(SubscriberManagerGuard::CurrentThread((shutdown, guard)))
    }
}

pub(crate) enum SubscriberManagerGuard {
    Global(Shutdown),
    CurrentThread((Shutdown, DefaultGuard)),
}

impl SubscriberManagerGuard {
    pub(crate) async fn shutdown(self) -> ShutdownResult<()> {
        match self {
            Self::Global(shutdown) => {
                shutdown().await?;
            }
            Self::CurrentThread((shutdown, guard)) => {
                shutdown().await?;
                drop(guard);
            }
        }
        Ok(())
    }
}

create_init_submodule! {
    classes: [
        PyConfig
    ],
}

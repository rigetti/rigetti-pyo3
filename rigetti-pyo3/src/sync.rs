//! Helpers that allow asynchronous Rust functions to be exported as synchronous Python functions.

use pyo3::prelude::*;
#[cfg(feature = "stubs")]
use pyo3_stub_gen::{PyStubType, TypeInfo};
#[cfg(feature = "async-tokio")]
use std::{
    ffi::CString,
    future::Future,
    sync::{mpsc, Arc, Mutex},
};
use std::{marker::PhantomData, sync::LazyLock};

/// The result of an asynchronous Python function.
///
/// When using `pyo3_async_runtimes`, functions that aren't meant to be `await`ed in Python
/// are created in Rust as synchronous with a return type of `Bound<'_, PyAny>`.
/// This type makes it clear that the function is in fact async (that it should be `await`ed),
/// and it provides access to the actual return type,
/// which enables a `PyStubType` implementation, and hence automatic stub generation.
///
/// # Example
///
/// ```
/// use pyo3::prelude::*;
/// use pyo3::py_run;
/// use pyo3_async_runtimes::tokio::future_into_py;
/// use rigetti_pyo3::sync::Awaitable;
///
/// # fn main() {
/// #[pyclass]
/// struct MyClass {
///     message: String,
/// }
///
/// #[pymethods]
/// impl MyClass {
///     fn get_message<'py>(&self, py: Python<'py>) -> PyResult<Awaitable<'py, String>> {
///         let msg = self.message.clone();
///         future_into_py(py, async move { Ok(msg) }).map(Into::into)
///     }
/// }
///
/// Python::initialize();
/// Python::attach(|py| {
///     let data = MyClass { message: "hello, world!".to_string() };
///     let data = Py::new(py, data).unwrap();
///     let MyClass = py.get_type::<MyClass>();
///
///     py_run!(py, data MyClass, r#"
/// import asyncio
///
/// async def check_message(inst: MyClass) -> None:
///     message = await inst.get_message()
///     assert message == "hello, world!"
///
/// asyncio.run(check_message(data))
///         "#);
/// })
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Awaitable<'py, T>(pub Bound<'py, PyAny>, PhantomData<T>);

impl<'py, T> Awaitable<'py, T> {
    /// Create a new `Awaitable` from a Python object.
    #[must_use]
    pub const fn new(obj: Bound<'py, PyAny>) -> Self {
        Awaitable(obj, PhantomData)
    }
}

impl<'py, T> FromPyObject<'_, 'py> for Awaitable<'py, T> {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        Ok(Awaitable(obj.to_owned(), PhantomData))
    }
}

impl<'py, T> IntoPyObject<'py> for Awaitable<'py, T> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, _: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0)
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a Awaitable<'py, T> {
    type Target = PyAny;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, _: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.as_borrowed())
    }
}

impl<'py, T> From<Bound<'py, PyAny>> for Awaitable<'py, T> {
    fn from(obj: Bound<'py, PyAny>) -> Self {
        Awaitable::new(obj)
    }
}

#[cfg(feature = "stubs")]
impl<T> PyStubType for Awaitable<'_, T>
where
    T: PyStubType,
{
    fn type_output() -> TypeInfo {
        let TypeInfo { name, mut import } = T::type_output();
        let name = format!("collections.abc.Awaitable[{name}]");
        import.insert("collections.abc".into());

        TypeInfo { name, import }
    }
}

#[cfg(feature = "async-tokio")]
/// Spawn and block on a future using the pyo3 tokio runtime.
/// Useful for returning a synchronous `PyResult`.
///
/// This is only a macro for backwards compatibility with older versions of the crate;
/// we can replace it with a function in a breaking-change release.
///
/// When used like the following:
/// ```rs
/// async fn say_hello(name: String) -> String {
///     format!("hello {name}")
/// }
///
/// #[pyo3(name="say_hello")]
/// pub fn py_say_hello(py: Python<'_>, name: String) -> PyResult<String> {
///     py_sync!(py, say_hello(name))
/// }
/// ```
///
/// Becomes the associated "synchronous" python call:
/// ```py
/// assert say_hello("Rigetti") == "hello Rigetti"
/// ```
#[macro_export]
macro_rules! py_sync {
    ($py:ident, $body:expr $(,)?) => {{
        $crate::sync::invoke_async_from_py_sync($body)
    }};
}

static PY_WORKER_EVENT_LOOP: LazyLock<Py<PyAny>> = LazyLock::new(|| {
    // Spawn a Python thread; start an event loop there.
    // Return a handle to the Python event loop.
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        Python::attach(|py| {
            let code = CString::new(
                r#"
import asyncio

# Create and set up the event loop
loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)

# Run the loop indefinitely (this will block the thread from completing until cancelled)
loop.run_until_complete(asyncio.sleep(float('inf')))
"#,
            )
            .unwrap();
            let module_name = CString::new("py_worker_event_loop").unwrap();

            let module = PyModule::from_code(py, &code, &module_name, &module_name)
                .expect("failed to create worker event loop module");

            // Get the loop variable from the module
            let loop_obj = module
                .getattr("loop")
                .expect("failed to get loop from module");

            // Send the loop to the channel
            let py_loop = loop_obj.into();
            let _ = tx.send(py_loop);
        })
    });

    // Wait for the worker thread to create and send the event loop
    rx.recv()
        .expect("failed to receive event loop from worker thread")
});

/// Spawn and block on a future using the pyo3 tokio runtime.
/// Useful for returning a synchronous `PyResult`.
///
/// This function is only necessary as a workaround for https://github.com/PyO3/pyo3-async-runtimes/issues/81.
///
#[cfg(feature = "async-tokio")]
pub async fn invoke_async_from_py_sync<F, T>(py: ::pyo3::Python<'_>, body: F) -> PyResult<T>
where
    F: Future<Output = PyResult<T>> + Send + 'static,
    T: Send + Sync + 'static,
{
    // Copied from `pyo3_async_runtimes`:
    let result_tx = Arc::new(Mutex::new(None));
    let result_rx = Arc::clone(&result_tx);
    let coro = ::pyo3_async_runtimes::tokio::future_into_py_with_locals(
        py,
        ::pyo3_async_runtimes::TaskLocals::new(PY_WORKER_EVENT_LOOP.bind(py).to_owned())
            .copy_context(py)?,
        async move {
            let val = body.await?;
            if let Ok(mut result) = result_tx.lock() {
                *result = Some(val);
            }
            Ok(())
        },
    )?;

    // Call `asyncio.run_coroutine_threadsafe` using `PY_WORKER_EVENT_LOOP`
    let run_coro_threadsafe = py.import("asyncio")?.getattr("run_coroutine_threadsafe")?;
    let concurrent_future =
        run_coro_threadsafe.call((&coro, PY_WORKER_EVENT_LOOP.bind(py)), None)?;

    // Block waiting for the result with a timeout
    let timeout_secs = 30.0;
    let _ = concurrent_future.call_method1("result", (timeout_secs,))?;

    let result = result_rx.lock().unwrap().take().unwrap();
    Ok(result)
}

#[cfg(feature = "async-tokio")]
/// Convert a rust future into a Python awaitable using
/// `pyo3_async_runtimes::tokio::future_into_py`
#[macro_export]
macro_rules! py_async {
    ($py:ident, $body:expr $(,)?) => {
        $crate::pyo3_async_runtimes::tokio::future_into_py($py, $body)
    };
}

/// Generate sync and async functions from a single implementation of an async function.
///
/// Given a single implementation of an async function,
/// create that function as private and two pyfunctions
/// named after it that can be used to invoke either
/// blocking or async variants of the same function.
///
/// In order to export the function to Python using pyo3
/// you must include the `#[pyfunction]` attribute. This
/// isn't included in the macro by default since one may
/// wish to annotate `#[pyfunction]` with additional
/// arguments.
///
/// The given function will be spawned on a Rust event loop
/// this means functions like [`pyo3::Python::with_gil`](pyo3::Python::with_gil)
/// should not be used, as acquiring Python's global
/// interpreter lock from a Rust runtime
/// isn't possible.
///
/// This macro cannot be used when lifetime specifiers are
/// required, or the pyfunction bodies need additional
/// parameter handling.
///
/// ```rs
/// // ... becomes python package "things"
/// create_init_submodule! {
///     funcs: [
///         py_do_thing,
///         py_do_thing_async,
///     ]
/// }
///
/// py_function_sync_async! {
///     #[pyfunction]
///     #[args(timeout = "None")]
///     async fn do_thing(timeout: Option<u64>) -> PyResult<String> {
///         // ... sleep for timeout ...
///         Ok(String::from("done"))
///     }
/// }
/// ```
///
/// becomes in python:
/// ```py
/// from things import do_thing, do_thing_async
/// assert do_thing() == "done"
/// assert await do_thing_async() == "done"
/// ```
///
/// With the `opentelemetry` feature enabled, this macro ensures Opentelemetry contexts are propagated.
#[macro_export]
macro_rules! py_function_sync_async {
    (
        $(#[$meta: meta])+
        $pub:vis async fn $name:ident($($(#[$arg_meta: meta])*$arg: ident : $kind: ty),* $(,)?)
        $(-> PyResult<$ret: ty>)? $body: block
    ) => {
        ::paste::paste! {
        async fn [< $name _impl >]($($arg: $kind,)*) $(-> PyResult<$ret>)? {
            $body
        }

        $(#[$meta])+
        #[allow(clippy::too_many_arguments)]
        #[pyo3(name = $name "")]
        $pub fn [< py_ $name >](py: $crate::pyo3::Python<'_> $(, $(#[$arg_meta])*$arg: $kind)*) $(-> PyResult<$ret>)? {
            let res = $crate::sync::add_context_if_otel([< $name _impl >]($($arg),*));
            $crate::py_sync!(py, res)
        }
        }

        $crate::py_function_sync_async! {
            @async_block {
                $(#[$meta])+
                $pub async fn $name($($(#[$arg_meta])*$arg : $kind),*) $(-> PyResult<$ret>)? $body
            }
        }
    };

    (
        @async_block {
            $(#[$meta: meta])+
            $pub:vis async fn $name:ident($($(#[$arg_meta: meta])*$arg: ident : $kind: ty),* $(,)?) $body: block
        }
    ) => {
        $crate::py_function_sync_async! {
            @async_block {
                $(#[$meta])+
                $pub async fn $name($($(#[$arg_meta])*$arg: $kind),*) -> () $body
            }
        };
    };

    (
        @async_block {
            $(#[$meta: meta])+
            $pub:vis async fn $name:ident($($(#[$arg_meta: meta])*$arg: ident : $kind: ty),* $(,)?)
            -> PyResult<$ret:ty> $body: block
        }
    ) => {
        ::paste::paste! {
        $(#[$meta])+
        #[pyo3(name = $name "_async")]
        #[allow(clippy::too_many_arguments)]
        $pub fn [< py_ $name _async >](py: $crate::pyo3::Python<'_> $(, $(#[$arg_meta])*$arg: $kind)*)
            -> ::pyo3::PyResult<$crate::sync::Awaitable<'_, $ret>>
        {
            let res = $crate::sync::add_context_if_otel([< $name _impl >]($($arg),*));
            $crate::pyo3_async_runtimes::tokio::future_into_py(py, res)
                .map($crate::sync::Awaitable::new)
        }
        }
    };
}

/// Adds a context, as the `opentelemetry` feature was enabled at build time.
#[cfg(feature = "opentelemetry")]
pub fn add_context_if_otel<T>(res: T) -> opentelemetry::trace::WithContext<T> {
    use opentelemetry::trace::FutureExt;
    res.with_current_context()
}

/// Acts as an identity function, as the `opentelemetry` feature was not enabled at build time.
#[cfg(not(feature = "opentelemetry"))]
#[inline]
pub const fn add_context_if_otel<T>(res: T) -> T {
    res
}

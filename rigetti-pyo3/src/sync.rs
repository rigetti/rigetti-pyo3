//! Helpers that allow asynchronous Rust functions to be exported as synchronous Python functions.

use pyo3::prelude::*;
#[cfg(feature = "stubs")]
use pyo3_stub_gen::{PyStubType, TypeInfo};
#[cfg(feature = "async-tokio")]
use std::{
    ffi::CString,
    future::Future,
    sync::{Arc, Mutex},
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
        $crate::sync::invoke_async_from_py_sync($py, $body)
    }};
}

/// The variable name to extract after executing `PY_CODE_WORKER_EVENT_LOOP` to get the worker event loop.
// This must match the name of the value defined in `PY_CODE_WORKER_EVENT_LOOP` below.
const PY_VARNAME_LOOP: &str = "loop";
/// The Python code snippet to set up a long-running background thread to run Rust futures from a
/// synchronous Python context.
/// This should only be run once per process.
const PY_CODE_WORKER_EVENT_LOOP: &str = r#"
import asyncio
import threading
import concurrent.futures

loop_fut = concurrent.futures.Future[asyncio.AbstractEventLoop]()

def _run_loop() -> None:
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    loop_fut.set_result(loop)
    loop.run_forever()

_thread = threading.Thread(
    target=_run_loop,
    name="rigetti-pyo3-worker-loop",
    # Daemon threads do not prevent the Python process from exiting.
    daemon=True,
)
_thread.start()
loop = loop_fut.result(timeout=1)
"#;

/// The function name to extract after executing `PY_CODE_RUN_ON_LOOP` to get the helper function
/// for running a coroutine on the worker event loop.
// This must match the name of the function defined in `PY_CODE_RUN_ON_LOOP` below.
const PY_FUNCNAME_RUN_ON_LOOP: &str = "get_result";
/// The Python code snippet to run a coroutine on the worker event loop and block until it returns
/// a result.
const PY_CODE_RUN_ON_LOOP: &str = r"
import asyncio

def get_result(loop, awaitable):
    async def _run_coroutine():
        return await awaitable
    
    return asyncio.run_coroutine_threadsafe(
        _run_coroutine(),
        loop,
    ).result()
";

/// Worker event loop used for running asynchronous tasks from synchronous Python code.
static PY_WORKER_EVENT_LOOP: LazyLock<Py<PyAny>> = LazyLock::new(|| {
    // Create a worker event loop and start it on a Python-created daemon thread.
    Python::attach(|py| {
        let code = CString::new(PY_CODE_WORKER_EVENT_LOOP).unwrap();
        let module_name = CString::new("py_worker_event_loop").unwrap();
        let module = PyModule::from_code(py, &code, &module_name, &module_name)
            .expect("failed to create worker event loop module");

        module
            .getattr(PY_VARNAME_LOOP)
            .expect("failed to get Python event loop variable from module")
            .into()
    })
});

/// Spawn and block on a future using the pyo3 tokio runtime.
/// Useful for returning a synchronous `PyResult`.
///
/// This function is only necessary as a workaround for <https://github.com/PyO3/pyo3-async-runtimes/issues/81>.
///
///
/// # Errors
///
/// Returns the result of the asynchronous operation, or an error if the operation fails, or if a
/// PyO3 failure prevents invoking the asynchronous operation.
///
/// # Panics
///
/// Panics if a lock has been poisoned or if certain string-literals are invalid C strings.
#[cfg(feature = "async-tokio")]
pub fn invoke_async_from_py_sync<F, T>(py: ::pyo3::Python<'_>, body: F) -> PyResult<T>
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

    // `future_into_py_with_locals` returns an awaitable; wrap it into a coroutine object
    // because `run_coroutine_threadsafe` requires a coroutine specifically.
    let code = CString::new(PY_CODE_RUN_ON_LOOP).unwrap();

    let module_name = CString::new("py_worker_event_loop_helper").unwrap();
    let module = PyModule::from_code(py, &code, &module_name, &module_name)?;

    let _ = module
        .getattr(PY_FUNCNAME_RUN_ON_LOOP)?
        .call((PY_WORKER_EVENT_LOOP.bind(py), &coro), None)?;

    let result = result_rx
        .lock()
        .unwrap()
        .take()
        .expect("future must always produce either a result or an error");
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
        $crate::paste! {
        async fn [< $name _impl >]($($arg: $kind,)*) $(-> PyResult<$ret>)? {
            $body
        }

        $(#[$meta])+
        #[allow(clippy::too_many_arguments)]
        #[pyo3(name = $name "")]
        $pub fn [< py_ $name >](py: $crate::pyo3::Python<'_> $(, $(#[$arg_meta])*$arg: $kind)*) $(-> PyResult<$ret>)? {
            let res = $crate::sync::add_context_if_otel([< $name _impl >]($($arg),*));
            $crate::sync::invoke_async_from_py_sync(py, res)
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
        $crate::paste! {
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

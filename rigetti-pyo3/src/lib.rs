// Copyright 2025 Rigetti Computing
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

//! Helpful macros and traits for creating a Python bindings to a Rust library.
//!
//! # Usage
//!
//! See the examples directory in the source for example usage of a majority of the crate.
//!
//! Alternatively, check the examples on the macros in this documentation.

// Covers correctness, suspicious, style, complexity, and perf
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![deny(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(clippy::nursery)]
// Conflicts with unreachable_pub
#![allow(clippy::redundant_pub_crate)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    bad_style,
    dead_code,
    keyword_idents,
    improper_ctypes,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
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
    unused_qualifications,
    variant_size_differences,
    while_true
)]

mod errors;
#[cfg(feature = "stubs")]
pub mod stubs;
#[cfg(feature = "async-tokio")]
pub mod sync;
mod traits;

#[cfg(feature = "async-tokio")]
pub use pastey::paste;

pub use pyo3;
#[cfg(feature = "async-tokio")]
pub use pyo3_async_runtimes;
#[cfg(feature = "stubs")]
pub use pyo3_stub_gen;
#[cfg(feature = "async-tokio")]
pub use tokio;

/// Create a crate-private function `init_submodule` to set up this submodule and call the same
/// function on child modules (which should also use this macro).
///
/// This generates boilerplate for exposing classes, exceptions, functions, and child modules to
/// the Python runtime, including a hack to allow importing from submodules, i.e.:
///
/// ```python,ignore
/// from foo.bar import baz
/// ```
///
/// # Example
///
/// ```
/// # fn main() {
/// use rigetti_pyo3::{create_init_submodule, exception, create_exception};
/// use rigetti_pyo3::pyo3::{prelude::*, exceptions::PyIOError};
///
/// #[pyfunction]
/// fn do_nothing() {}
///
/// #[pyclass]
/// struct CoolString(String);
///
/// #[derive(Debug, thiserror::Error)]
/// #[error("io error: {0}")]
/// struct RustIOError(#[from] std::io::Error);
///
/// exception!(RustIOError, "example", IOError, PyIOError, "IO Error");
///
/// mod my_submodule {
///     use rigetti_pyo3::create_init_submodule;
///     use rigetti_pyo3::pyo3::pyclass;
///
///     #[pyclass]
///     struct CoolInt(i32);
///
///     create_init_submodule! {
///         classes: [ CoolInt ],
///     }
/// }
///
/// create_init_submodule! {
///     /// Initialize this module and all its submodules
///     classes: [ CoolString ],
///     errors: [ IOError ],
///     funcs: [ do_nothing ],
///     submodules: [ "my_submodule": my_submodule::init_submodule ],
/// }
///
/// #[pymodule]
/// fn example(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
///     init_submodule("example", py, m)
/// }
/// # }
/// ```
#[macro_export]
macro_rules! create_init_submodule {
    (
        $(#[$meta:meta])*
        $(classes: [ $($class: ty),+  $(,)? ],)?
        $(complex_enums: [ $($complex_enum: ty),+ $(,)? ],)?
        $(consts: [ $($const: ident),+ $(,)? ],)?
        $(errors: [ $($error: ty),+ $(,)? ],)?
        $(funcs: [ $($func: path),+ $(,)? ],)?
        $(submodules: [ $($mod_name: literal: $init_submod: path),+ $(,)? ],)?
    ) => {
        $(#[$meta])*
        pub(crate) fn init_submodule<'py>(_name: &str, _py: $crate::pyo3::Python<'py>, m: &$crate::pyo3::Bound<'py, $crate::pyo3::types::PyModule>) -> $crate::pyo3::PyResult<()> {
            $($(
            $crate::pyo3::types::PyModuleMethods::add_class::<$class>(m)?;
            )+)?
            $($(
            $crate::pyo3::types::PyModuleMethods::add_class::<$complex_enum>(m)?;
            )+)?
            $($(
                $crate::pyo3::types::PyModuleMethods::add(m,
                    ::std::stringify!($const),
                    $crate::pyo3::IntoPyObject::into_pyobject(&$const, _py)?
                )?;
            )+)?
            $($(
                $crate::pyo3::types::PyModuleMethods::add(m,
                    $crate::pyo3::types::PyTypeMethods::name(&_py.get_type::<$error>())?,
                    _py.get_type::<$error>()
                )?;
            )+)?
            $($(
            $crate::pyo3::types::PyModuleMethods::add_function(m, $crate::pyo3::wrap_pyfunction!($func, m)?)?;
            )+)?
            $(
                let sys = $crate::pyo3::types::PyModule::import(_py, "sys")?;
                let modules = $crate::pyo3::types::PyAnyMethods::getattr(sys.as_any(), "modules")?;
                $(
                let qualified_name = format!("{}.{}", _name, $mod_name);
                let submod = $crate::pyo3::types::PyModule::new(_py, $mod_name)?;
                $init_submod(&qualified_name, _py, &submod)?;
                $crate::pyo3::types::PyModuleMethods::add_submodule(m, &submod)?;
                $crate::pyo3::types::PyAnyMethods::set_item(modules.as_any(), &qualified_name, &submod)?;
                )+
            )?
            Ok(())
        }
    }
}

/// This ensures that our enums are pickleable.
/// There was a bug prior to PyO3 0.27 that caused enums not to be pickleable
/// until we modified the `__qualname__` attribute.
#[cfg(test)]
mod test_pickle_roundtrip {
    use pyo3::types::{PyDict, PyTuple};
    use pyo3::{prelude::*, py_run};

    #[pyclass(module = "mymod")]
    enum Foo {
        Integer { value: i64 },
        Real { value: f64 },
    }

    #[pymethods]
    impl Foo {
        fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
            match self {
                Self::Integer { value } => PyTuple::new(py, [value]),
                Self::Real { value } => PyTuple::new(py, [value]),
            }
        }
    }

    #[pyclass(module = "mymod")]
    enum Bar {
        Integer(i64),
        Real(f64),
    }

    #[pymethods]
    impl Bar {
        fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
            match self {
                Self::Integer(value) => PyTuple::new(py, [value]),
                Self::Real(value) => PyTuple::new(py, [value]),
            }
        }
    }

    #[pymodule(name = "mymod")]
    fn mymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<Foo>()?;
        m.add_class::<Bar>()?;

        Ok(())
    }

    /// Verify that we can pickle and unpickle complex enums
    #[test]
    fn test_enum_pickle_roundtrip() {
        pyo3::append_to_inittab!(mymod);
        Python::initialize();
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py_run!(
                py,
                *locals,
                r#"
import pickle
import mymod
from mymod import Foo

# All enums should be picklable.
objs = [
    Foo.Integer(42),
    Foo.Real(3.14),
    mymod.Bar.Integer(42),
    mymod.Bar.Real(3.14),
]

for obj in objs:
    result = pickle.loads(pickle.dumps(obj))
    match obj:
        case Foo.Integer(value=x) | Foo.Real(value=x):
            assert result.value == x
        case mymod.Bar.Integer(x) | mymod.Bar.Real(x):
            assert result._0 == x
        case _:
            raise TypeError(f"Unexpected object: {obj}")
"#
            );
        });
    }
}

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

pub use pyo3;
#[cfg(feature = "async-tokio")]
pub use pyo3_async_runtimes;
#[cfg(feature = "stubs")]
pub use pyo3_stub_gen;
#[cfg(feature = "async-tokio")]
pub use tokio;

use pyo3::{
    prelude::*,
    types::{PyList, PyTuple, PyType, PyTypeMethods},
};

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
/// use rigetti_pyo3::{create_init_submodule, exception, create_exception};
/// use rigetti_pyo3::pyo3::{prelude::*, exceptions::PyException};
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
/// exception!(RustIOError, "example", IOError, PyException, "IO Error");
///
/// mod my_submodule {
///     use rigetti_pyo3::{create_init_submodule};
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
/// ```
#[macro_export]
macro_rules! create_init_submodule {
    (
        $(#[$meta:meta])*
        $(classes: [ $($class: ty),+ ],)?
        $(complex_enums: [ $($complex_enum: ty),+ ],)?
        $(consts: [ $($const: ident),+ ],)?
        $(errors: [ $($error: ty),+ ],)?
        $(funcs: [ $($func: path),+ ],)?
        $(submodules: [ $($mod_name: literal: $init_submod: path),+ ],)?
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
            $($(
            $crate::fix_enum_qual_names(&_py.get_type::<$complex_enum>())?;
            )+)?
            Ok(())
        }
    }
}

/// Fix the `__qualname__` on PyO3's "complex enums" so that they can be pickled.
///
/// Essentially, this runs the following Python code:
///
/// ```python
/// import inspect
/// issubclass = lambda cls: inspect.isclass(cls) and issubclass(cls, typ)
/// for name, cls in inspect.getmembers(typ, issubclass):
///     cls.__qualname__ = f"{prefix}.{name}"
/// ```
///
/// # In a Pickle
///
/// PyO3 processes `enum`s with non-unit variants by creating a Python class for the enum,
/// then creating a class for each variant, subclassed from the main enum class.
/// The subclasses end up as attributes on the main enum class,
/// which enables syntax like `q = Qubit.Fixed(0)`;
/// however, they're given qualified names that use `_` as a seperator instead of `.`,
/// e.g. we get `Qubit.Fixed(0).__qualname__ == "Qubit_Fixed"`
/// rather than `Qubit.Fixed`, as we would if we had written the inner class ourselves.
/// As a consequence, attempting to `pickle` an instance of it
/// will raise an error complaining that `quil.instructions.Qubit_Fixed` can't be found.
///
/// There are a handful of ways of making this work,
/// but modifying the `__qualname__` seems not only simple, but correct.
///
/// # See Also
///
/// - PyO3's Complex Enums: <https://pyo3.rs/v0.25.1/class#complex-enums>
/// - Issue regarding `__qualname__`: <https://github.com/PyO3/pyo3/issues/5270>
/// - Python's `inspect`: <https://docs.python.org/3/library/inspect.html#inspect.getmembers>
pub fn fix_enum_qual_names(typ: &Bound<'_, PyType>) -> PyResult<()> {
    let py = typ.py();
    let inspect = PyModule::import(py, "inspect")?;
    let isclass = inspect.getattr("isclass")?;
    let get_members = inspect.getattr("getmembers")?;

    let prefix = typ.qualname()?;
    let prefix = prefix.as_borrowed();
    let prefix = prefix.to_str()?;

    let inner: Bound<'_, PyList> = get_members.call((typ, isclass), None)?.cast_into()?;
    for item in &inner {
        let item = item.cast::<PyTuple>()?;

        let cls = item.get_borrowed_item(1)?;
        if cls.cast()?.is_subclass(typ)? {
            // See https://pyo3.rs/v0.25.1/types#borroweda-py-t for info on `get_borrowed_item`.
            let name = item.get_borrowed_item(0)?;
            let fixed_name = format!("{prefix}.{}", name.cast()?.to_str()?);
            cls.setattr(pyo3::intern!(py, "__qualname__"), fixed_name)?;
        }
    }

    Ok(())
}

/// Fix the `__qualname__` on a list of complex enums so that they can be pickled.
/// See [`fix_enum_qual_names`] for more information.
///
/// The first argument should be a `Python<'py>` instance;
/// all others should be names of `#[pyclass]`-annotated `enum`s with non-unit variants.
///
/// # Example
///
/// ```ignore
/// use pyo3;
/// use pyo3_stub_gen::derive::gen_stub_pyclass_complex_enum;
///
/// #[pyo3::pymodule(name = "place", module = "some", submodule)]
/// fn init_some_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
///   let py = m.py();
///
///   m.add_class::<Foo>()?;
///   m.add_class::<Bar>()?;
///
///   fix_complex_enums!(py, Foo, Bar);
/// }
///
/// #[gen_stub_pyclass_complex_enum]
/// #[pyo3::pyclass(module = "some.place", eq, frozen, hash, get_all)]
/// pub enum Foo {
///     Integer(i64),
///     Real(f64),
/// }
///
/// #[gen_stub_pyclass_complex_enum]
/// #[pyo3::pyclass(module = "some.place", eq, frozen, hash, get_all)]
/// pub enum Bar {
///     Integer(i64),
///     Real(f64),
/// }
/// ```
#[macro_export]
macro_rules! fix_complex_enums {
    ($py:expr, $($name:path),* $(,)?) => {
        {
            let py = $py;
            $($crate::fix_enum_qual_names(&py.get_type::<$name>())?;)*
        }
    };
}

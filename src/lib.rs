// Copyright 2022 Rigetti Computing
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

//! Helpful macros and traits for creating a Python wrapper of a Rust library.
//!
//! See [Macros](#macros) and [Traits](#traits) for the main items in this crate.
//!
//! # Usage
//!
//! See the examples directory in the source for example usage of a majority of the crate.
//!
//! Alternatively, check the examples on the macros in this documentation.

// Covers correctness, suspicious, style, complexity, and perf
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
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
    pointer_structural_match,
    private_in_public,
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

use pyo3::PyErr;

#[cfg(feature = "time")]
pub mod datetime;
mod py_try_from;
mod to_python;
mod traits;
mod wrappers;

#[cfg(feature = "complex")]
pub use num_complex;
pub use paste;
pub use py_try_from::PyTryFrom;
pub use pyo3;
pub use to_python::ToPython;

/// Implemented by error types generated with [`py_wrap_error`](crate::py_wrap_error).
///
/// Trait-ifies the ability to convert an error into a [`PyErr`](crate::pyo3::PyErr).
pub trait ToPythonError {
    /// Convert this error into a [`PyErr`](crate::pyo3::PyErr).
    fn to_py_err(self) -> PyErr;
}

impl ToPythonError for PyErr {
    fn to_py_err(self) -> PyErr {
        self
    }
}

impl ToPythonError for std::convert::Infallible {
    fn to_py_err(self) -> PyErr {
        unreachable!("Infallible can never happen")
    }
}

/// Implemented by wrapper types generated with `py_wrap_*` macros:
///
/// - [`py_wrap_struct`]
/// - [`py_wrap_union_enum`]
/// - [`py_wrap_simple_enum`]
/// - [`py_wrap_type`]
pub trait PyWrapper: From<Self::Inner> + Into<Self::Inner> + AsRef<Self::Inner> {
    /// The Rust type being wrapped.
    type Inner;

    /// Returns a reference to the inner item.
    ///
    /// Like [`AsRef`], but doesn't require generics.
    fn as_inner(&self) -> &Self::Inner {
        self.as_ref()
    }

    /// Converts this into the inner item.
    ///
    /// Like [`Into`], but doesn't require generics.
    fn into_inner(self) -> Self::Inner {
        self.into()
    }
}

/// Implemented by wrapper types containing the source type, generated with `py_wrap_*` macros:
///
/// - [`py_wrap_struct`]
/// - [`py_wrap_union_enum`]
/// - [`py_wrap_type`]
///
/// The notable exception is [`py_wrap_simple_enum`], where it does not make sense to have a mutable
/// reference to a unit enum.
pub trait PyWrapperMut: PyWrapper + AsMut<Self::Inner> {
    /// Returns a mutable reference to the inner item.
    ///
    /// Like [`AsMut`], but doesn't require generics.
    fn as_inner_mut(&mut self) -> &mut Self::Inner {
        self.as_mut()
    }
}

impl<T> PyWrapperMut for T where T: PyWrapper + AsMut<Self::Inner> {}

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
/// use rigetti_pyo3::{py_wrap_type, py_wrap_error, wrap_error, create_init_submodule};
/// use rigetti_pyo3::pyo3::{pyfunction, pymodule, Python, PyResult, types::PyModule};
/// use rigetti_pyo3::pyo3::exceptions::PyRuntimeError;
///
/// #[pyfunction]
/// fn do_nothing() {}
///
/// py_wrap_type! {
///     PyCoolString(String) as "CoolString";
/// }
///
/// wrap_error!{
///     RustIOError(std::io::Error);
/// }
///
/// py_wrap_error!(errors, RustIOError, IOError, PyRuntimeError);
///
/// mod my_submodule {
///     use rigetti_pyo3::{py_wrap_type, create_init_submodule};
///     
///     py_wrap_type! {
///         PyCoolInt(i32) as "CoolInt";
///     }
///
///     create_init_submodule! {
///         classes: [ PyCoolInt ],
///     }
/// }
///
/// create_init_submodule! {
///     classes: [ PyCoolString ],
///     errors: [ IOError ],
///     funcs: [ do_nothing ],
///     submodules: [ "my_submodule": my_submodule::init_submodule ],
/// }
///
/// #[pymodule]
/// fn example(py: Python<'_>, m: &PyModule) -> PyResult<()> {
///     init_submodule("example", py, m)
/// }
/// ```
#[macro_export]
macro_rules! create_init_submodule {
    (
        $(classes: [ $($class: ty),+ ],)?
        $(consts: [ $($const: ident),+ ],)?
        $(errors: [ $($error: ty),+ ],)?
        $(funcs: [ $($func: path),+ ],)?
        $(submodules: [ $($mod_name: literal: $init_submod: path),+ ],)?
    ) => {
        pub(crate) fn init_submodule(_name: &str, _py: $crate::pyo3::Python, m: &$crate::pyo3::types::PyModule) -> $crate::pyo3::PyResult<()> {
            $($(
            m.add_class::<$class>()?;
            )+)?
            $($(
            m.add(::std::stringify!($const), $crate::ToPython::<$crate::pyo3::Py<$crate::pyo3::PyAny>>::to_python(&$const, _py)?)?;
            )+)?
            $($(
            m.add(std::stringify!($error), _py.get_type::<$error>())?;
            )+)?
            $($(
            m.add_function($crate::pyo3::wrap_pyfunction!($func, m)?)?;
            )+)?
            $(
                let modules = _py.import("sys")?.getattr("modules")?;
                $(
                let submod = $crate::pyo3::types::PyModule::new(_py, &$mod_name)?;
                let qualified_name = format!("{}.{}", _name, $mod_name);
                $init_submod(&qualified_name, _py, submod)?;
                modules.set_item(qualified_name, submod)?;
                m.add_submodule(submod)?;
                )+
            )?
            Ok(())
        }
    }
}

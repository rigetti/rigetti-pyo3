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

//! Macros for implementing trait-related behavior for wrapped types.
//!
//! - Implement Rust traits based on inner types.
//! - Implement "dunder" methods based on wrapper type trait implementations.
//!   - Note that you can pass `#[derive(Trait)]` as part of the `py_wrap_*` macro input

/// Implement Python comparison for a given type. That type must implement
/// [`PartialOrd`](std::cmp::PartialOrd).
#[macro_export]
macro_rules! impl_compare {
    ($name: ident) => {
        #[$crate::pyo3::pymethods]
        impl $name {
            #![allow(clippy::use_self)]
            pub fn __richcmp__(&self, object: &Self, cmp: $crate::pyo3::basic::CompareOp) -> bool {
                let result = ::std::cmp::PartialOrd::partial_cmp(self, object);
                match cmp {
                    $crate::pyo3::basic::CompareOp::Lt => {
                        matches!(result, Some(::std::cmp::Ordering::Less))
                    }
                    $crate::pyo3::basic::CompareOp::Le => {
                        !matches!(result, Some(::std::cmp::Ordering::Greater))
                    }
                    $crate::pyo3::basic::CompareOp::Eq => {
                        matches!(result, Some(::std::cmp::Ordering::Equal))
                    }
                    $crate::pyo3::basic::CompareOp::Ne => {
                        !matches!(result, Some(::std::cmp::Ordering::Equal))
                    }
                    $crate::pyo3::basic::CompareOp::Gt => {
                        matches!(result, Some(::std::cmp::Ordering::Greater))
                    }
                    $crate::pyo3::basic::CompareOp::Ge => {
                        !matches!(result, Some(::std::cmp::Ordering::Less))
                    }
                }
            }
        }
    };
}

/// Implement `__hash__` for types that implement [`Hash`](std::hash::Hash).
#[macro_export]
macro_rules! impl_hash {
    ($name: ident) => {
        #[$crate::pyo3::pymethods]
        impl $name {
            pub fn __hash__(&self) -> i64 {
                let mut hasher = ::std::collections::hash_map::DefaultHasher::new();
                ::std::hash::Hash::hash($crate::PyWrapper::as_inner(self), &mut hasher);
                let bytes = ::std::hash::Hasher::finish(&hasher).to_ne_bytes();
                i64::from_ne_bytes(bytes)
            }
        }
    };
}

/// Implement `__repr__` for wrapper types whose inner type implements [`Debug`](std::fmt::Debug).
#[macro_export]
macro_rules! impl_repr {
    ($name: ident) => {
        #[$crate::pyo3::pymethods]
        impl $name {
            pub fn __repr__(&self) -> String {
                format!("{:?}", $crate::PyWrapper::as_inner(self))
            }
        }
    };
}

/// Implement `__str__` for wrapper types whose inner type implements [`Display`](std::fmt::Display).
#[macro_export]
macro_rules! impl_str {
    ($name: ident) => {
        #[$crate::pyo3::pymethods]
        impl $name {
            pub fn __str__(&self) -> String {
                format!("{}", $crate::PyWrapper::as_inner(self))
            }
        }
    };
}

/// Implement [`FromStr`](std::str::FromStr) for wrapper types whose inner type implements [`FromStr`](std::str::FromStr).
///
/// The second argument must be a Python error wrapper that implements [`From<E>`], where `E = <$name::Inner as FromStr>::Err`.
#[macro_export]
macro_rules! impl_from_str {
    ($name: ident, $error: ty) => {
        impl ::std::str::FromStr for $name {
            type Err = $error;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                <<Self as $crate::PyWrapper>::Inner as ::std::str::FromStr>::from_str(input)
                    .map(Self::from)
                    .map_err(
                        <$error as From<
                            <<Self as $crate::PyWrapper>::Inner as ::std::str::FromStr>::Err,
                        >>::from,
                    )
            }
        }
    };
}

/// Implement a method `parse` for wrapper types that implement [`FromStr`](std::str::FromStr).
///
/// See also: [`impl_from_str`].
#[macro_export]
macro_rules! impl_parse {
    ($name: ident) => {
        #[$crate::pyo3::pymethods]
        impl $name {
            #[staticmethod]
            pub fn parse(input: &str) -> $crate::pyo3::PyResult<Self> {
                <Self as std::str::FromStr>::from_str(input)
                    .map(Self::from)
                    .map_err($crate::ToPythonError::to_py_err)
            }
        }
    };
}

/// Implement `AsMut<T>` for a Python wrapper around `T`.
///
/// This macro is automatically invoked by [`py_wrap_struct`](crate::py_wrap_struct)
/// and [`py_wrap_union_enum`](crate::py_wrap_union_enum), and should otherwise only be used if
/// you need more flexibility and are using [`py_wrap_type`](crate::py_wrap_type) directly.
#[macro_export]
macro_rules! impl_as_mut_for_wrapper {
    ($py_type: ident) => {
        impl AsMut<<$py_type as $crate::PyWrapper>::Inner> for $py_type {
            fn as_mut(&mut self) -> &mut <$py_type as $crate::PyWrapper>::Inner {
                &mut self.0
            }
        }
    };
}

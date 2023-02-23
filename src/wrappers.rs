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

//! Macros for wrapping different Rust types for use in Python.

/// Creates a new exception type and implements converting from the given Rust error to the new
/// exception.
///
/// The Rust error type must at least implement [`ToString`](std::string::ToString). All types
/// that implement [`Error`](std::error::Error) implement this through
/// [`Display`](std::fmt::Display).
///
///
/// ```
/// use rigetti_pyo3::py_wrap_error;
/// use rigetti_pyo3::pyo3::exceptions::PyValueError;
/// use std::fmt;
///
/// #[derive(Debug)]
/// enum RustError {}
///
/// impl fmt::Display for RustError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         unimplemented!()
///     }
/// }
///
/// impl std::error::Error for RustError {}
///
/// py_wrap_error!(my_python_module, RustError, PythonError, PyValueError);
/// ```
#[macro_export]
macro_rules! py_wrap_error {
    ($module: ident, $rust: ty, $python: ident, $base: ty) => {
        $crate::pyo3::create_exception!($module, $python, $base);

        impl $crate::ToPythonError for $rust {
            fn to_py_err(self) -> $crate::pyo3::PyErr {
                <$python>::new_err(self.to_string())
            }
        }
    };
}

/// Create a Python wrapper around a Rust type.
///
/// You probably do not want to call this directly, as other macros build on top of this.
///
/// Implements:
/// - Conversion between wrapper and inner Rust type
/// - `AsRef` to access the inner Rust type from [`pyo3`](crate::pyo3) code.
/// - [`PyWrapper`](crate::PyWrapper) as non-generic aliases for the above
/// - [`ToPyObject`](crate::pyo3::conversion::ToPyObject)
///
/// # Macro inputs:
///
/// - `$meta`: Any attributes to apply to the wrapper type. Supports `#[pyo3(...)]`
///   for configuring the Python type.
/// - `$name`: The Rust name for the wrapper type (usually `PySomething`).
/// - `$from`: The Rust type to wrap.
/// - `$py_alias` (optional): The type name to expose to Python (usually `$name` without a leading `Py`).
/// - `$pyclass_param` (optional): A list of parameters to pass to #[pyclass(...)]. The `name`
/// parameter should not be used as it overlaps with `$py_alias`.
/// - `$pyclass_value` (optional): A value associated with a `$pyclass_param`.
///
/// ```
/// use std::collections::HashMap;
/// use rigetti_pyo3::py_wrap_type;
///
/// py_wrap_type! {
///     #[derive(Debug)]
///     PyNumberLabels(HashMap<String, i32>) as "NumberLabels";
/// }
///
/// let map = HashMap::new();
/// let dict = PyNumberLabels::from(map);
/// let map = HashMap::from(dict);
/// let dict = PyNumberLabels::from(&map);
/// assert_eq!(&map, dict.as_ref());
/// ```
#[macro_export]
macro_rules! py_wrap_type {
    (
        $(#[$meta: meta])*
        $name: ident($from: ty)$(as $py_alias: literal)? $(with $( $pyclass_meta: ident $(= $pyclass_value: literal)?),+ )?$(;)?
    ) => {
        #[repr(transparent)]
        #[allow(clippy::use_self)]
        #[$crate::pyo3::pyclass($($($pyclass_meta $(= $pyclass_value)?),+ , )?$(name = $py_alias)?)]
        #[derive(Clone)]
        $(#[$meta])*
        pub struct $name($from);

        impl $crate::PyTryFrom<$name> for $from {
            fn py_try_from(
                py: $crate::pyo3::Python,
                item: &$name,
            ) -> $crate::pyo3::PyResult<Self> {
                Ok(item.0.clone())
            }
        }

        impl $crate::PyTryFrom<$crate::pyo3::PyAny> for $name {
            fn py_try_from(
                py: $crate::pyo3::Python,
                item: &$crate::pyo3::PyAny,
            ) -> $crate::pyo3::PyResult<Self> {
                item.extract()
            }
        }

        impl $crate::PyTryFrom<$name> for $name {
            fn py_try_from(
                py: $crate::pyo3::Python,
                item: &$name,
            ) -> $crate::pyo3::PyResult<Self> {
                Ok(item.clone())
            }
        }

        $crate::private_impl_to_python_with_reference!(&self, py, $from => $name {
            Ok($name::from(self.clone()))
        });

        $crate::private_impl_to_python_with_reference!(&self, py, $name => $crate::pyo3::Py<$crate::pyo3::PyAny> {
            Ok(<Self as $crate::pyo3::ToPyObject>::to_object(self, py))
        });

        impl From<$name> for $from {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }

        impl From<$from> for $name {
            fn from(inner: $from) -> Self {
                Self(inner)
            }
        }

        impl From<&$from> for $name {
            fn from(inner: &$from) -> Self {
                Self(inner.clone())
            }
        }

        impl AsRef<$from> for $name {
            fn as_ref(&self) -> &$from {
                &self.0
            }
        }

        impl $crate::PyWrapper for $name {
            type Inner = $from;
        }

        impl $crate::pyo3::conversion::ToPyObject for $name {
            fn to_object(&self, py: $crate::pyo3::Python) -> $crate::pyo3::PyObject {
                #[allow(clippy::use_self)]
                const NAME: &'static str = stringify!($name);
                let cell = $crate::pyo3::PyCell::new(py, self.clone())
                    .unwrap_or_else(|err| {
                        panic!(
                            "failed to create {} on Python heap: {}",
                            NAME,
                            err
                        )
                    });
                $crate::pyo3::conversion::ToPyObject::to_object(&cell, py)
            }
        }
    };
}

/// Wrap an enum containing only unit variants.
///
/// Implements
///
/// - Conversion between Rust and Python types (also converting from references to each)
///
/// # Macro Inputs
///
/// - `$variant_name`: comma-separated list of variant names on the Rust enum. Required because
///   there is no way to do reflection to programmatically find them.
/// - `$variant_alias`: used in conjunction with `$variant_name` as the Python enum member name
///   when it should be named differently, useful in cases when the enum member name is not a
///   valid python identifier. If one variant uses an alias, they all must, even if the alias
///   is the same as the name.
/// - See also [`py_wrap_type`].
///
/// # Example
///
/// ```
/// use rigetti_pyo3::py_wrap_simple_enum;
///
/// #[derive(Copy, Clone)]
/// pub enum RustEnum {
///     Foo,
///     Bar,
/// }
///
/// py_wrap_simple_enum! {
///     PyEnum(RustEnum) {
///         Foo,
///         Bar
///     }
/// }
///
/// py_wrap_simple_enum! {
///     PyEnumAliased(RustEnum) {
///         Foo as FOO,
///         Bar as Bar
///     }
/// }
/// ```
#[macro_export]
macro_rules! py_wrap_simple_enum {
    (
        $(#[$meta: meta])*
        $name: ident($rs_inner: ident) $(as $py_class: literal)? {
            $($variant_name: ident),+
        }
    ) => {
        $crate::py_wrap_simple_enum! {
            $(#[$meta])*
            $name($rs_inner) $(as $py_class)? {
                $($variant_name as $variant_name),+
            }
        }
    };
    (
        $(#[$meta: meta])*
        $name: ident($rs_inner: ident) $(as $py_class: literal)? {
            $($variant_name: ident as $variant_alias: ident),+
        }
    ) => {
        #[derive(Copy, Clone)]
        #[$crate::pyo3::pyclass$((name = $py_class))?]
        $(#[$meta])*
        pub enum $name {
            $(
            $variant_alias
            ),+
        }

        impl From<$name> for $rs_inner {
            fn from(item: $name) -> Self {
                match item {
                    $(
                    $name::$variant_alias => Self::$variant_name,
                    )+
                }
            }
        }

        impl From<&$name> for $rs_inner {
            fn from(item: &$name) -> Self {
                Self::from(*item)
            }
        }

        impl From<$rs_inner> for $name {
            fn from(item: $rs_inner) -> Self {
                match item {
                    $(
                    $rs_inner::$variant_name => $name::$variant_alias,
                    )+
                }
            }
        }

        impl From<&$rs_inner> for $name {
            fn from(item: &$rs_inner) -> Self {
                Self::from(*item)
            }
        }

        impl $crate::PyWrapper for $name {
            type Inner = $rs_inner;
        }

        impl AsRef<$rs_inner> for $name {
            fn as_ref(&self) -> &$rs_inner {
                match self {
                    $(
                    $name::$variant_alias => &$rs_inner::$variant_name,
                    )+
                }
            }
        }

        impl $crate::pyo3::conversion::ToPyObject for $name {
            fn to_object(&self, py: $crate::pyo3::Python) -> $crate::pyo3::PyObject {
                let cell = $crate::pyo3::PyCell::new(py, self.clone())
                    .unwrap_or_else(|err| panic!("failed to create {} on Python heap: {}", stringify!($name), err));
                cell.to_object(py)
            }
        }

        $crate::private_impl_to_python_with_reference!(&self, _py, $rs_inner => $name {
            Ok($name::from(self))
        });

        $crate::private_impl_py_try_from!(&item, _py, $name => $rs_inner {
            Ok(*item.as_ref())
        });
    }
}

/// Create a newtype wrapper for a Rust struct.
///
/// Implements the following:
///
/// - Conversion to/from the contained Rust type
/// - Conversion to/from the related Python/Rust types
/// - Constructor taking any type that can be converted from
///
/// # Limitations
///
/// This macro generates a `__new__` constructor for the Python type from the given
/// `py -> rs` conversions. This constructor expects exactly one parameter, which cannot
/// be omitted (i.e. has no default value).
///
/// To have more control over the constructor, use [`py_wrap_type`] with a manual
/// implementation in a `pymethods` `impl` block.
///
/// # Example
///
/// ```
/// use rigetti_pyo3::py_wrap_struct;
/// use rigetti_pyo3::pyo3::{Py, PyErr, Python};
/// use rigetti_pyo3::pyo3::conversion::{IntoPy, PyTryFrom, ToPyObject};
/// use rigetti_pyo3::pyo3::types::{PyDict, PyTuple};
///
/// #[derive(Clone)]
/// pub struct Foo {
///     bar: String,
///     baz: f32,
/// }
///
/// impl From<(String, f32)> for Foo {
///     fn from(tuple: (String, f32)) -> Self {
///         Self { bar: tuple.0, baz: tuple.1 }
///     }
/// }
///
/// impl From<Foo> for (String, f32) {
///     fn from(foo: Foo) -> Self {
///         (foo.bar, foo.baz)
///     }
/// }
///
/// py_wrap_struct! {
///     PyFoo(Foo) {
///         // Fallible transformation from Python type `P` to Rust type `T` where `Foo: From<T>`.
///         // Used to implement `TryFrom<P> for PyFoo`. Any errors returned must be `PyErr`.
///         py -> rs {
///             py_dict: Py<PyDict> => Foo {
///                 let bar = py_dict.as_ref(py).get_item("bar").unwrap().extract().unwrap();
///                 let baz = py_dict.as_ref(py).get_item("baz").unwrap().extract().unwrap();
///                 Ok::<_, PyErr>(Foo { bar, baz })
///             },
///             py_tuple: Py<PyTuple> => (String, f32) {
///                 Ok::<_, PyErr>((
///                     py_tuple.as_ref(py).get_item(0).unwrap().extract().unwrap(),
///                     py_tuple.as_ref(py).get_item(1).unwrap().extract().unwrap(),
///                 ))
///             }
///         },
///         // Infallible transformation from Rust type `T` to Python type `P` where `T: From<Foo>`.
///         // Used to implement `From<PyFoo> for P`.
///         rs -> py {
///             rs_tuple: (String, f32) => Py<PyTuple> {
///                 Python::with_gil(|py| {
///                     let obj = rs_tuple.to_object(py);
///                     <PyTuple as PyTryFrom>::try_from(obj.as_ref(py))
///                         .map(|tuple| tuple.into_py(py))
///                         .map_err(PyErr::from)
///                 })
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! py_wrap_struct {
    (
        $(#[$meta: meta])*
        $name: ident($rs_from: ty) $(as $py_class: literal)? $(with $( $pyclass_meta: ident$( = $pyclass_value: literal)?),+ )? {
            /// Fallible transformation from Python type `P` to Rust type `T` where `Foo: From<T>`.
            /// Used to implement `TryFrom<P> for PyFoo`. Any errors returned must be `PyErr`.
            ///
            /// $py_for_from should conventionally be `py` -- it is the name of the `Python<'_>` parameter.
            $($py_for_from: ident -> rs {
                $($py_ident: ident: $py_src: ty => $rs_dest: ty $to_rs: block),+
            },)?
            /// Fallible transformation from Rust type `T` to Python type `P` where `T: From<Foo>`
            /// Used to implement `TryFrom<PyFoo> for P`. Any errors returned must be `PyErr`.
            ///
            /// $py_for_to should conventionally be `py` -- it is the name of the `Python<'_>` parameter.
            $(rs -> $py_for_to: ident {
                $($rs_ident: ident: $rs_src: ty => $py_dest: ty $to_py: block),+
            })?
        }
    ) => {
        $crate::py_wrap_type! {
            $(
            #[$meta]
            )*
            $name($rs_from) $(as $py_class)? $(with $($pyclass_meta $( = $pyclass_value)?),+ )?;
        }

        $($(
        impl TryFrom<$py_src> for $name {
            #[allow(unused_qualifications)]
            type Error = pyo3::PyErr;
            fn try_from($py_ident: $py_src) -> Result<Self, Self::Error> {
                $crate::pyo3::Python::with_gil(|$py_for_from| {
                    let rust = {
                        $to_rs
                    }?;
                    Ok(Self::from(<$rs_from>::from(rust)))
                })
            }
        }
        )+)?

        $($(
        impl TryFrom<$name> for $py_dest {
            #[allow(unused_qualifications)]
            type Error = pyo3::PyErr;
            fn try_from(outer: $name) -> Result<Self, Self::Error> {
                let $rs_ident = $crate::PyWrapper::into_inner(outer);
                let $rs_ident: $rs_src = From::from($rs_ident);
                $crate::pyo3::Python::with_gil(|$py_for_to| {
                    $to_py
                })
            }
        }
        )+)?

        $crate::impl_as_mut_for_wrapper!($name);

        #[$crate::pyo3::pymethods]
        impl $name {
            #![allow(clippy::use_self)]

            #[new]
            pub fn new(py: $crate::pyo3::Python, input: $crate::pyo3::Py<$crate::pyo3::PyAny>) -> $crate::pyo3::PyResult<Self> {
                use $crate::pyo3::FromPyObject;

                $($(
                if let Ok(item) = input.extract::<$py_src>(py) {
                    return Self::try_from(item);
                }
                )+)?

                Err($crate::pyo3::exceptions::PyValueError::new_err(
                    concat!("expected one of:" $($(, " ", std::stringify!($py_src))+)?)
                ))
            }
        }
    }
}

/// (Internal) Helper macro to get the final type in a chain of conversions.
///
/// Necessary because the pattern `$(=> $foo: ty)* => $bar: ty` is ambiguous.
#[macro_export]
macro_rules! private_ultimate_type {
    ($type: ty) => { $type };
    ($type: ty, $($others: ty),+) => { $crate::private_ultimate_type!($($others),+) }
}

/// (Internal) Helper macro to implement chained conversion through intermediate types,
/// where the type system cannot determine a path from the first to last item.
#[macro_export]
macro_rules! private_intermediate_to_python {
    ($py: ident, &$item: ident $(=> $convert: ty)+) => {{
        $(
        let $item: $convert = $crate::ToPython::<$convert>::to_python(&$item, $py)?;
        )+
        Ok::<_, $crate::pyo3::PyErr>($item)
    }}
}

/// (Internal) Helper macro to implement chained conversion through intermediate types,
/// where the type system cannot determine a path from the last to first item.
#[macro_export]
macro_rules! private_intermediate_try_from_python {
    ($py: ident, &$item: ident => $convert: ty $($(=> $delayed: ty)+)?) => {{
        $(let $item: $convert = $crate::private_intermediate_try_from_python!($py, &$item $(=> $delayed)+)?;
        let $item = &$item;)?
        <_ as $crate::PyTryFrom<$convert>>::py_try_from($py, $item)
    }};
}

/// Create a newtype wrapper for a Rust enum with unique 1-tuple variants.
///
/// # Implements
///
/// - Conversion between the wrapper and the inner enum
/// - A Python constructor that creates a new instance from one of the Python variants.
/// - A Python function `inner()` that directly returns the Python version of the variant
///   discriminant (i.e. `Discriminant` in `Enum::Variant(Discriminant)`).
/// - Python conversion functions:
///     - `from_x`: Like the constructor, but for a specific variant `x`.
///     - `is_x`: Returns `True` if the enum is variant `x`.
///     - `as_x`: Returns the discriminant if the enum is variant `x`, otherwise `None`.
///     - `to_x`: Returns the discriminant if the enum is variant `x`, otherwise raises
///       `ValueError`.
///
/// # Example
///
/// ```
/// use rigetti_pyo3::py_wrap_union_enum;
/// use rigetti_pyo3::pyo3::prelude::*;
/// use rigetti_pyo3::pyo3::types::*;
/// use std::collections::HashSet;
///
/// #[derive(Clone)]
/// pub enum TestEnum {
///     Unit,
///     String(String),
///     Integer(i32),
///     UInteger(u32),
///     List(Vec<HashSet<String>>),
///     Mapping(std::collections::HashMap<String, String>),
/// }
///
/// py_wrap_union_enum! {
///     PyTestEnum(TestEnum) as "TestEnum" {
///         // Syntax is (1): (2) [=> (3)] [=> (4)] [...], where:
///         // 1: The name used in generated methods
///         // 2: The name of the Rust enum variant
///         // 3: The (Python) type the inner item must convert to (if it has associated data)
///         // 4: The (Python) type the type from (3) must convert to, etc.
///         unit: Unit,
///         // Read as "give the name `string` to variant `String`, which must convert (from
///         // a `String`) to a `String` and then to a `Py<PyString>`."
///         //
///         // That is, `string` is used to generate methods `is_string`, `from_string`, etc.;
///         // the first `String` is the name of the variant, not the type (which is elided);
///         // the second `String` is the type to convert the elided type into, and `Py<String>` is
///         // the final type to convert into.
///         //
///         // This specifies an unnecessary conversion from String => String to illustrate
///         // conversion chaining.
///         string: String => String => Py<PyString>,
///         int: Integer => Py<PyInt>,
///         uint: UInteger => Py<PyInt>,
///         list: List => Py<PyList>,
///         // Alternatively, in the case of `Vec<T>` where `T` does not have conversion to `PyAny`.
///         // list: List => Vec<Py<PySet>> => Py<PyList>,
///         // Generates `from_dict`, `is_dict`, `as_dict`, `to_dict`
///         dict: Mapping => Py<PyDict>
///     }
/// }
/// ```
#[macro_export]
macro_rules! py_wrap_union_enum {
    // @from creates its own impl block to avoid an error of "cannot find attribute `staticmethod`
    // in this scope".
    //
    // There may be a performance hit to this, but I (@Shadow53) cannot figure out how to do this
    // otherwise without rewriting everything as procedural macros.
    //
    // Note: the cause of the error was the use of `paste!` *within* a `pymethods` impl block.
    // Having the impl block within the `paste!` macro is what makes the error go away.
    (@from $name: ident, $rs_enum: ident, $variant_name: ident, $variant: ident $(=> $convert: ty)+) => {
        $crate::paste::paste! {
            #[$crate::pyo3::pymethods]
            impl $name {
                #[staticmethod]
                pub fn [< from_ $variant_name >](py: $crate::pyo3::Python, inner: $crate::private_ultimate_type!($($convert),+)) -> $crate::pyo3::PyResult<Self> {
                    let inner = &inner;
                    $crate::private_intermediate_try_from_python!(py, &inner $(=> $convert)+)
                        .map($rs_enum::$variant)
                        .map(Self)
                }
            }
        }
    };
    (@from $name: ident, $rs_enum: ident, $variant_name: ident, $variant: ident) => {
        $crate::paste::paste! {
            #[$crate::pyo3::pymethods]
            impl $name {
                #[staticmethod]
                pub fn [< new_ $variant_name >]() -> Self {
                    Self::from($rs_enum::$variant)
                }
            }
        }
    };
    (@is_variant $self: ident, $rs_enum: ident, $variant: ident ($(=> $_convert: ty)+)) => {
        match &$self.0 {
            $rs_enum::$variant(_) => true,
            _ => false,
        }
    };
    (@is_variant $self: ident, $rs_enum: ident, $variant: ident) => {
        match &$self.0 {
            $rs_enum::$variant => true,
            _ => false,
        }
    };
    (
        $(#[$meta: meta])*
        $name: ident($rs_inner: ident) $(as $py_class: literal)? $(with $( $pyclass_meta: ident$( = $pyclass_value: literal)?),+ )? {
            $($variant_name: ident: $variant: ident $($(=> $convert: ty)+)?),+
        }
    ) => {
        $crate::py_wrap_type! {
            $(#[$meta])*
            $name($rs_inner) $(as $py_class)? $(with $($pyclass_meta $( = $pyclass_value)?),+ )?;
        }

        $crate::impl_as_mut_for_wrapper!($name);

        $(
        $crate::py_wrap_union_enum!(@from $name, $rs_inner, $variant_name, $variant $($(=> $convert)+)?);
        )+

        $crate::paste::paste! {
        #[$crate::pyo3::pymethods]
        impl $name {
            #[new]
            pub fn new(py: $crate::pyo3::Python, input: &$crate::pyo3::PyAny) -> $crate::pyo3::PyResult<Self> {
                $(
                    $(
                        if let Ok(inner) = <_ as $crate::PyTryFrom<$crate::pyo3::PyAny>>::py_try_from(py, input) {
                            let inner = &inner;
                            let converted = $crate::private_intermediate_try_from_python!(py, &inner $(=> $convert)+);
                            if let Ok(item) = converted {
                                return Ok(Self::from($rs_inner::$variant(item)));
                            }
                        }
                    )?
                )+

                Err($crate::pyo3::exceptions::PyValueError::new_err(
                    format!(
                        "could not create {} from {}",
                        stringify!($name),
                        input.repr()?
                    )
                ))
            }

            #[allow(unreachable_code, unreachable_pattern)]
            pub fn inner(&self, py: $crate::pyo3::Python) -> $crate::pyo3::PyResult<$crate::pyo3::Py<$crate::pyo3::PyAny>> {
                match &self.0 {
                    $(
                        $($rs_inner::$variant(inner) => {
                            Ok($crate::pyo3::conversion::IntoPy::<$crate::pyo3::Py<$crate::pyo3::PyAny>>::into_py(
                                $crate::private_intermediate_to_python!(py, &inner $(=> $convert)+)?,
                                py,
                            ))
                        },)?
                    )+
                    _ => {
                        use $crate::pyo3::exceptions::PyRuntimeError;
                        Err(PyRuntimeError::new_err("Enum variant has no inner data or is unimplemented"))
                    }
                }
            }

            $(
            const fn [< is_ $variant_name >](&self) -> bool {
                $crate::py_wrap_union_enum!(@is_variant self, $rs_inner, $variant $(($(=> $convert)+))?)
            }

                $(
                fn [< as_ $variant_name >](&self, py: $crate::pyo3::Python) -> Option<$crate::private_ultimate_type!($($convert),+)> {
                    self.[< to_ $variant_name >](py).ok()
                }

                fn [< to_ $variant_name >](&self, py: $crate::pyo3::Python) -> $crate::pyo3::PyResult<$crate::private_ultimate_type!($($convert),+)> {
                    if let $rs_inner::$variant(inner) = &self.0 {
                        $crate::private_intermediate_to_python!(py, &inner $(=> $convert)+)
                    } else {
                        Err($crate::pyo3::exceptions::PyValueError::new_err(
                            concat!("expected self to be a ", stringify!($variant_name))
                        ))
                    }
                }
                )?
            )+
        }
        }
    }
}

/// Wraps an external error type in a newtype `struct` so it can be used with [`py_wrap_error`].
///
/// # Implements
///
/// - [`From`] impls between the newtype and the inner type.
/// - [`Display`](std::fmt::Display) delegating to the inner type
/// - [`Error`](std::error::Error)
///
/// # Example
///
/// ```
/// use rigetti_pyo3::{wrap_error, py_wrap_error};
/// use rigetti_pyo3::pyo3::exceptions::PyRuntimeError;
///
/// wrap_error!{
///     RustIOError(std::io::Error);
/// }
///
/// py_wrap_error!(errors, RustIOError, IOError, PyRuntimeError);
/// ```
#[macro_export]
macro_rules! wrap_error {
    ($name: ident ($inner: ty)$(;)?) => {
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $name($inner);

        impl From<$inner> for $name {
            fn from(inner: $inner) -> Self {
                Self(inner)
            }
        }

        impl From<$name> for $inner {
            fn from(outer: $name) -> Self {
                outer.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl ::std::error::Error for $name {}
    };
}

/// Wraps a data struct and makes (some of) its fields available to Python.
///
/// # Implements
///
/// - Everything implemented by [`py_wrap_type`].
/// - [`PyWrapperMut`](crate::PyWrapperMut).
/// - `get_foo` and `set_foo` methods for field `foo`, which translate to `@property` and
///   `@foo.setter` in Python, i.e. allowing access to the field as a property.
///
/// # Warning!
///
/// The mutability of exposed fields may not work as you expect.
///
/// Since objects are converted back and forth along the FFI boundary using `Clone`s,
/// pointers are not shared like in native Python. In native Python, this code runs
/// without issue:
///
/// ```python
/// class Test:
///   def __init__(self):
///       self.inner = {}
///
///   @property
///   def foo(self):
///     return self.inner
///
///   @foo.setter
///   def foo(self, value):
///     self.inner = value
///
/// c = Test()
/// d = c.inner
///
/// d["a"] = ["a"]
/// assert "a" in c.inner
///
/// d = c.foo
/// d["b"] = "b"
/// assert "b" in c.inner
/// ```
///
/// Using these bindings, assuming that this macro was used to create `Test`, the
/// equivalent would be:
///
/// ```python
/// c = Test()
/// d = c.foo
///
/// d["a"] = ["a"]
/// assert "a" not in c.foo
/// c.foo = d
/// assert "a" in c.foo
/// ```
///
/// # Example
///
/// ```
/// use rigetti_pyo3::pyo3::{Py, types::{PyInt, PyString}};
/// use rigetti_pyo3::py_wrap_data_struct;
///
/// #[derive(Clone)]
/// pub struct Person {
///     pub name: String,
///     pub age: u8,
/// }
///
/// py_wrap_data_struct! {
///     PyPerson(Person) as "Person" {
///         name: String => Py<PyString>,
///         age: u8 => Py<PyInt>
///     }
/// }
/// ```
#[macro_export]
macro_rules! py_wrap_data_struct {
    (
        $(#[$meta: meta])*
        $name: ident($rs_inner: ty) $(as $class_name: literal)? $(with $( $pyclass_meta: ident$( = $pyclass_value: literal)?),+ )? {
            $(
            $field_name: ident: $field_rs_type: ty $(=> $convert: ty)+
            ),+
        }
    ) => {
        $crate::py_wrap_type! {
            $(
            #[$meta]
            )*
            $name($rs_inner) $(as $class_name)? $(with $($pyclass_meta $( = $pyclass_value)?),+ )?;
        }

        $crate::impl_as_mut_for_wrapper!($name);

        $crate::paste::paste! {
            #[rigetti_pyo3::pyo3::pymethods]
            impl $name {
                $(
                #[getter]
                fn [< get_ $field_name >](&self, py: $crate::pyo3::Python<'_>) -> $crate::pyo3::PyResult<$crate::private_ultimate_type!($($convert),+)> {
                    use $crate::{PyWrapper, ToPython};
                    let inner = &self.as_inner().$field_name;
                    $crate::private_intermediate_to_python!(py, &inner $(=> $convert)+)
                }

                #[setter]
                fn [< set_ $field_name >](&mut self, py: $crate::pyo3::Python<'_>, from: $crate::private_ultimate_type!($($convert),+)) -> $crate::pyo3::PyResult<()> {
                    use $crate::{PyTryFrom, PyWrapperMut};
                    let from = &from;
                    let new_val: $field_rs_type = $crate::private_intermediate_try_from_python!(py, &from $(=> $convert)+)?;
                    self.as_inner_mut().$field_name = new_val;
                    Ok(())
                }
                )+
            }
        }
    };
}

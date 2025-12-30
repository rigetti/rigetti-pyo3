//! Macros for Python exceptions from Rust errors with correct stub types.

#[cfg(not(feature = "stubs"))]
/// Create a new Python exception.
#[macro_export]
macro_rules! create_exception {
    ( $module:expr, $py_err: ident, $base: ty ) => {
        create_exception!($module, $py_err, $base, "");
    };
    ( $module:expr, $py_err: ident, $base: ty, $doc: expr ) => {
        $crate::pyo3::create_exception!($module, $py_err, $base, $doc);
    };
}

#[cfg(feature = "stubs")]
/// Create a new Python exception using the correct macro
/// based on whether a "stubs" features is active.
#[macro_export]
macro_rules! create_exception {
    ( $module:expr, $py_err: ident, $base: ty ) => {
        create_exception!($module, $py_err, $base, "");
    };
    ( $module:expr, $py_err: ident, $base: ty, $doc: expr ) => {
        $crate::pyo3::create_exception!($module, $py_err, $base, $doc);

        #[cfg(feature = "stubs")]
        impl $crate::pyo3_stub_gen::PyStubType for $py_err {
            fn type_output() -> $crate::pyo3_stub_gen::TypeInfo {
                $crate::pyo3_stub_gen::TypeInfo::locally_defined(
                    stringify!($py_err),
                    stringify!($module).into(),
                )
            }
        }

        #[cfg(feature = "stubs")]
        $crate::pyo3_stub_gen::inventory::submit! {
            $crate::pyo3_stub_gen::type_info::PyClassInfo {
                pyclass_name: stringify!($py_err),
                struct_id: ::std::any::TypeId::of::<$py_err>,
                getters: &[],
                setters: &[],
                module: Some(stringify!($module)),
                doc: $doc,
                bases: &[|| <$base as $crate::pyo3_stub_gen::PyStubType>::type_output()],
                has_eq: false,
                has_ord: false,
                has_hash: false,
                has_str: false,
                subclass: true,
            }
        }
    };
}

/// Create a Python exception and a conversion from its Rust type.
/// Note that the exception class must still be added to the module.
#[macro_export]
macro_rules! exception {
    ( $rust_err: ty, $module:expr, $py_err: ident, $base: ty $(, $doc: expr)? ) => {
        create_exception!( $module, $py_err, $base $(, $doc)? );

        #[doc = concat!(
            "Convert a Rust ",
            "`", stringify!($rust_err), "`",
            " into a Python ",
            "`", stringify!($py_err), "`."
        )]
        impl ::std::convert::From<$rust_err> for $crate::pyo3::PyErr {
            fn from(err: $rust_err) -> Self {
                $py_err::new_err(err.to_string())
            }
        }
    };
}

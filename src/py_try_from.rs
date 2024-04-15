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

//! Unifying conversion traits from Python to Rust data.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use pyo3::types::PyComplex;
use pyo3::{
    exceptions::PyValueError,
    types::{PyDate, PyDateTime, PyDelta, PyTime, PyTzInfo},
};
use pyo3::{
    types::{
        PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyFrozenSet, PyInt, PyList, PySet, PyString,
    },
    FromPyObject, IntoPy, Py, PyAny, PyResult, Python,
};

#[cfg(feature = "complex")]
use num_complex::Complex;
#[cfg(feature = "complex")]
use num_traits::{Float, FloatConst};
#[cfg(feature = "complex")]
use pyo3::exceptions::PyFloatingPointError;
#[cfg(feature = "complex")]
use std::fmt::Display;
#[cfg(feature = "complex")]
use std::os::raw::c_double;

#[cfg(feature = "time")]
use crate::datetime::DateTime;
#[cfg(feature = "time")]
use pyo3::{types::PyTuple, ToPyObject};
#[cfg(feature = "time")]
use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

#[cfg(feature = "indexmap")]
use indexmap::IndexMap;

/// Convert from a Python type to a Rust type.
pub trait PyTryFrom<P>: Sized {
    /// Convert from a `Py<T>`. Defaults to delegating to `py_from_ref`.
    ///
    /// # Errors
    ///
    /// Any errors that may occur during conversion.
    fn py_try_from(py: Python, item: &P) -> PyResult<Self>;
}

impl<P> PyTryFrom<PyAny> for Py<P>
where
    Self: for<'a> FromPyObject<'a>,
{
    fn py_try_from(_py: Python, item: &PyAny) -> PyResult<Self> {
        item.extract()
    }
}

impl<T, P> PyTryFrom<P> for Box<T>
where
    T: PyTryFrom<P>,
{
    fn py_try_from(py: Python, item: &P) -> PyResult<Self> {
        T::py_try_from(py, item).map(Self::new)
    }
}

/// Provides a generic implementation of [`PyTryFrom`] for heterogenous tuples of types that themselves implement [`PyTryFrom`]
macro_rules! impl_py_try_from_tuple {
    ($($idx:tt $t:tt $p:tt),+) => {
        impl<$($t,)+ $($p,)+> PyTryFrom<($($p,)+)> for ($($t,)+)
        where
            $($t: PyTryFrom<$p>,)+
        {
            fn py_try_from(py: Python, item: &($($p,)+)) -> PyResult<Self> {
                Ok(($(
                    $t :: py_try_from(py, &item.$idx)?,
                )+))
            }
        }
    };
}

// Implement [`PyTryFrom`] for tuples of length 1 to 12, 12 being the maximum arity that [`pyo3::ToPyObject`]
// is implemented for.
impl_py_try_from_tuple!(0 T0 P0);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8, 9 T9 P9);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8, 9 T9 P9, 10 T10 P10);
impl_py_try_from_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8, 9 T9 P9, 10 T10 P10, 11 T11 P11);

/// Provides a body for `py_try_from`, delegating to the implementation for the given Python type.
///
/// This should be used in other macros and for generic/container types that can't be implemented
/// entirely with a macro.
#[macro_export]
macro_rules! private_py_try_from_py_pyany_inner {
    ($item: ident, $py: ident, $py_type: ty) => {{
        let actual: &$py_type = $item.extract()?;
        <Self as $crate::PyTryFrom<$py_type>>::py_try_from($py, actual)
    }};
}

/// Generate `PyTryFrom` implementations for `PyAny` that require the `PyAny` to contain a specific Python type.
#[macro_export]
macro_rules! private_impl_py_try_from_pyany {
    ($py_type: ty => $rs_type: ty) => {
        impl $crate::PyTryFrom<$crate::pyo3::PyAny> for $rs_type {
            fn py_try_from(
                py: $crate::pyo3::Python,
                item: &$crate::pyo3::PyAny,
            ) -> $crate::pyo3::PyResult<Self> {
                let actual: &$py_type = item.extract()?;
                <Self as $crate::PyTryFrom<$py_type>>::py_try_from(py, actual)
            }
        }
    };
}

/// Implement [`PyTryFrom`] for a given Rust type.
#[macro_export]
#[allow(clippy::module_name_repetitions)]
macro_rules! private_impl_py_try_from {
    (&$item: ident, $py: ident, $py_type: ty => $rs_type: ty $convert: block) => {
        #[allow(clippy::use_self)]
        impl $crate::PyTryFrom<$py_type> for $rs_type {
            fn py_try_from(
                $py: $crate::pyo3::Python,
                $item: &$py_type,
            ) -> $crate::pyo3::PyResult<Self> {
                $convert
            }
        }
    };
}

/// Implement [`PyTryFrom<PyAny>`] for a given Rust type by delegating to its implementation
/// for the given Python type.
#[macro_export]
macro_rules! private_impl_py_try_from_with_pyany {
    (&$item: ident, $py: ident, $py_type: ty => $rs_type: ty $convert: block) => {
        $crate::private_impl_py_try_from!(&$item, $py, $py_type => $rs_type $convert);
        $crate::private_impl_py_try_from_pyany!($py_type => $rs_type);
    };
}

/// Implements [`PyTryFrom<Py<P>>`] for `T` where `T: PyTryFrom<P>` and `P` is a native Python type.
macro_rules! impl_try_from_py_native {
    ($py_type: ty => $rs_type: ty) => {
        private_impl_py_try_from!(&item, py, $crate::pyo3::Py<$py_type> => $rs_type {
            let item: &$py_type = item.as_ref(py);
            <Self as $crate::PyTryFrom<$py_type>>::py_try_from(py, item)
        });
    }
}

/// Implements [`PyTryFrom<T>`] for a `T` that is a native Python type.
macro_rules! impl_try_from_self_python {
    ($py_type: ty) => {
        private_impl_py_try_from!(&item, py, $py_type => $crate::pyo3::Py<$py_type> {
            Ok(item.into_py(py))
        });
        private_impl_py_try_from!(&item, _py, $crate::pyo3::Py<$py_type> => $crate::pyo3::Py<$py_type> {
            Ok(item.clone())
        });
    }
}

/// Implements [`PyTryFrom<T>`] for a `T` that is a native Rust type.
macro_rules! impl_try_from_self_rust {
    ($rs_type: ty) => {
        private_impl_py_try_from!(&item, _py, $rs_type => $rs_type {
            Ok(item.clone())
        });
    }
}

/// Implements [`PyTryFrom`] for primitive types by just calling `extract()`.
macro_rules! impl_try_from_primitive {
    ($py_type: ty => $rs_type: ty) => {
        private_impl_py_try_from_with_pyany!(&item, _py, $py_type => $rs_type { item.extract() });
        impl_try_from_py_native!($py_type => $rs_type);
    };
}

// ============ Begin Implementations ==============

// ==== Bool ====

impl_try_from_primitive!(PyBool => bool);
impl_try_from_self_python!(PyBool);
impl_try_from_self_rust!(bool);

// ==== ByteArray ====

impl_try_from_self_python!(PyByteArray);
impl_try_from_py_native!(PyByteArray => Vec<u8>);
private_impl_py_try_from!(&item, _py, PyByteArray => Vec<u8> {
    Ok(item.to_vec())
});

impl_try_from_py_native!(PyByteArray => Box<[u8]>);
private_impl_py_try_from!(&item, py, PyByteArray => Box<[u8]> {
    Vec::<u8>::py_try_from(py, item).map(From::from)
});

// ==== Bytes ====

impl_try_from_self_python!(PyBytes);
impl_try_from_py_native!(PyBytes => Vec<u8>);
private_impl_py_try_from!(&item, _py, PyBytes => Vec<u8> {
    Ok(item.as_bytes().to_vec())
});

impl_try_from_py_native!(PyBytes => Box<[u8]>);
private_impl_py_try_from!(&item, py, PyBytes => Box<[u8]> {
    Vec::<u8>::py_try_from(py, item).map(From::from)
});

// ==== Complex ====

impl_try_from_self_python!(PyComplex);

#[cfg(feature = "complex")]
impl<F> PyTryFrom<Self> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(_py: Python, item: &Self) -> PyResult<Self> {
        Ok(*item)
    }
}

#[cfg(feature = "complex")]
impl<F> PyTryFrom<Py<PyComplex>> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(py: Python, item: &Py<PyComplex>) -> PyResult<Self> {
        Self::py_try_from(py, item.as_ref(py))
    }
}

#[cfg(feature = "complex")]
impl<F> PyTryFrom<PyComplex> for Complex<F>
where
    // `Display` seems like an odd trait to require, but it is used to make a more useful
    // error message. The types realistically used for this are `f32` and `f64` both of which
    // impl `Display`, so there's no issue there.
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(_py: Python, item: &PyComplex) -> PyResult<Self> {
        let make_error = |val: c_double| {
            PyFloatingPointError::new_err(format!(
                "expected {val} to be between {} and {}, inclusive",
                F::min_value(),
                F::max_value(),
            ))
        };
        Ok(Self {
            re: F::from(item.real()).ok_or_else(|| make_error(item.real()))?,
            im: F::from(item.imag()).ok_or_else(|| make_error(item.imag()))?,
        })
    }
}

#[cfg(feature = "complex")]
impl<F> PyTryFrom<PyAny> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyComplex = item.downcast()?;
        Self::py_try_from(py, dict)
    }
}

// ==== Date ====

impl_try_from_self_python!(PyDate);

#[cfg(feature = "time")]
impl_try_from_self_rust!(Date);
#[cfg(feature = "time")]
impl_try_from_py_native!(PyDate => Date);

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyDate => Date {
    let year = item.getattr("year").map(|any| i32::py_try_from(py, any))??;
    let month: u8 = item.getattr("month").map(|any| u8::py_try_from(py, any))??; // 1-12
    let month: Month = month.try_into()
        .map_err(|_| {
            PyValueError::new_err(format!("Expected date month to be within 0-12, got {month}"))
        })?;
    let day = item.getattr("day").map(|any| u8::py_try_from(py, any))??; // 1-X

    Self::from_calendar_date(year, month, day).map_err(|err| {
        PyValueError::new_err(format!("Failed to create Date object: {err}"))
    })
});

// ==== DateTime ====

impl_try_from_self_python!(PyDateTime);

#[cfg(feature = "time")]
impl_try_from_self_rust!(DateTime);
#[cfg(feature = "time")]
impl_try_from_py_native!(PyDateTime => DateTime);

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyDateTime => DateTime {
    let date = item.call_method0("date").map(|date| Date::py_try_from(py, date))??;
    let (time, offset) = item.call_method0("timetz")
        .map(|time| <(Time, Option<UtcOffset>)>::py_try_from(py, time))??;
    let datetime = PrimitiveDateTime::new(date, time);
    let datetime = offset.map_or(Self::Primitive(datetime), |offset| {
            // Cannot create an OffsetDateTime from parts, for some reason.
            let datetime = OffsetDateTime::now_utc()
                .replace_date_time(datetime)
                .replace_offset(offset);
            Self::Offset(datetime)
        });

    Ok(datetime)
});

// ==== Delta ====

impl_try_from_self_python!(PyDelta);

#[cfg(feature = "time")]
impl_try_from_self_rust!(Duration);

#[cfg(feature = "time")]
impl_try_from_py_native!(PyDelta => Duration);

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, _py, PyDelta => Duration {
    let days: i64 = item.getattr("days")?.extract()?;
    let seconds: i64 = item.getattr("seconds")?.extract()?;
    let microseconds: i32 = item.getattr("microseconds")?.extract()?;
    let nanoseconds = microseconds.checked_mul(1000).ok_or_else(|| {
        PyValueError::new_err("Could not fit {microseconds} microseconds as nanoseconds into a 32-bit signed integer")
    })?;
    let day_seconds = days.checked_mul(24 * 60 * 60).ok_or_else(|| {
        PyValueError::new_err("Could not fit {days} days as seconds into a 64-bit signed integer")
    })?;
    let seconds = seconds.checked_add(day_seconds).ok_or_else(|| {
        PyValueError::new_err("Could not add {days} days and {seconds} seconds into a 64-bit signed integer")
    })?;
    Ok(Self::new(seconds, nanoseconds))
});

impl_try_from_self_rust!(std::time::Duration);
impl_try_from_py_native!(PyDelta => std::time::Duration);

private_impl_py_try_from!(&item, _py, PyDelta => std::time::Duration {
    let days: u64 = item.getattr("days")?.extract()?;
    let seconds: u64 = item.getattr("seconds")?.extract()?;
    let microseconds: u32 = item.getattr("microseconds")?.extract()?;
    let nanoseconds = microseconds.checked_mul(1000).ok_or_else(|| {
        PyValueError::new_err("Could not fit {microseconds} microseconds as nanoseconds into a 32-bit signed integer")
    })?;
    let day_seconds = days.checked_mul(24 * 60 * 60).ok_or_else(|| {
        PyValueError::new_err("Could not fit {days} days as seconds into a 64-bit signed integer")
    })?;
    let seconds = seconds.checked_add(day_seconds).ok_or_else(|| {
        PyValueError::new_err("Could not add {days} days and {seconds} seconds into a 64-bit signed integer")
    })?;
    Ok(Self::new(seconds, nanoseconds))
});

// ==== Dict ====

impl_try_from_self_python!(PyDict);

impl<K1, K2, V1, V2, Hasher> PyTryFrom<HashMap<K1, V1>> for HashMap<K2, V2, Hasher>
where
    K2: Eq + std::hash::Hash + PyTryFrom<K1>,
    V2: PyTryFrom<V1>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &HashMap<K1, V1>) -> PyResult<Self> {
        item.iter()
            .map(|(key, val)| {
                let key = K2::py_try_from(py, key)?;
                let val = V2::py_try_from(py, val)?;
                Ok((key, val))
            })
            .collect()
    }
}

impl<K, V, Hasher> PyTryFrom<Py<PyDict>> for HashMap<K, V, Hasher>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &Py<PyDict>) -> PyResult<Self> {
        Self::py_try_from(py, item.as_ref(py))
    }
}

impl<K, V, Hasher> PyTryFrom<PyDict> for HashMap<K, V, Hasher>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &PyDict) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(item.len(), Hasher::default());
        for (key, val) in item {
            let key = K::py_try_from(py, key)?;
            let val = V::py_try_from(py, val)?;
            map.insert(key, val);
        }
        Ok(map)
    }
}

impl<K, V, Hasher> PyTryFrom<PyAny> for HashMap<K, V, Hasher>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyDict = item.downcast()?;
        Self::py_try_from(py, dict)
    }
}

impl<K1, K2, V1, V2> PyTryFrom<BTreeMap<K1, V1>> for BTreeMap<K2, V2>
where
    K2: Ord + PyTryFrom<K1>,
    V2: PyTryFrom<V1>,
{
    fn py_try_from(py: Python, item: &BTreeMap<K1, V1>) -> PyResult<Self> {
        item.iter()
            .map(|(key, val)| {
                let key = K2::py_try_from(py, key)?;
                let val = V2::py_try_from(py, val)?;
                Ok((key, val))
            })
            .collect()
    }
}

impl<K, V> PyTryFrom<Py<PyDict>> for BTreeMap<K, V>
where
    K: Ord + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: &Py<PyDict>) -> PyResult<Self> {
        Self::py_try_from(py, item.as_ref(py))
    }
}

impl<K, V> PyTryFrom<PyDict> for BTreeMap<K, V>
where
    K: Ord + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: &PyDict) -> PyResult<Self> {
        let mut map = Self::new();
        for (key, val) in item {
            let key = K::py_try_from(py, key)?;
            let val = V::py_try_from(py, val)?;
            map.insert(key, val);
        }
        Ok(map)
    }
}

impl<K, V> PyTryFrom<PyAny> for BTreeMap<K, V>
where
    K: Ord + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyDict = item.downcast()?;
        <Self as PyTryFrom<PyDict>>::py_try_from(py, dict)
    }
}

// ==== Float ====

impl_try_from_self_python!(PyFloat);
impl_try_from_self_rust!(f32);
impl_try_from_self_rust!(f64);
impl_try_from_primitive!(PyFloat => f32);
impl_try_from_primitive!(PyFloat => f64);

// ==== FrozenSet ====

impl_try_from_self_python!(PyFrozenSet);

impl<T, Hasher> PyTryFrom<Py<PyFrozenSet>> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, set: &Py<PyFrozenSet>) -> PyResult<Self> {
        Self::py_try_from(py, set.as_ref(py))
    }
}

impl<T, Hasher> PyTryFrom<PyFrozenSet> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, set: &PyFrozenSet) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(set.len(), Hasher::default());
        for item in set {
            let item = T::py_try_from(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

impl<T> PyTryFrom<Py<PyFrozenSet>> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, set: &Py<PyFrozenSet>) -> PyResult<Self> {
        Self::py_try_from(py, set.as_ref(py))
    }
}

impl<T> PyTryFrom<PyFrozenSet> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, set: &PyFrozenSet) -> PyResult<Self> {
        let mut map = Self::new();
        for item in set {
            let item = T::py_try_from(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

// ==== Integer ====

impl_try_from_self_python!(PyInt);
impl_try_from_self_rust!(i8);
impl_try_from_self_rust!(i16);
impl_try_from_self_rust!(i32);
impl_try_from_self_rust!(i64);
impl_try_from_self_rust!(i128);
impl_try_from_self_rust!(isize);
impl_try_from_self_rust!(u8);
impl_try_from_self_rust!(u16);
impl_try_from_self_rust!(u32);
impl_try_from_self_rust!(u64);
impl_try_from_self_rust!(u128);
impl_try_from_self_rust!(usize);
impl_try_from_primitive!(PyInt => i8);
impl_try_from_primitive!(PyInt => i16);
impl_try_from_primitive!(PyInt => i32);
impl_try_from_primitive!(PyInt => i64);
impl_try_from_primitive!(PyInt => i128);
impl_try_from_primitive!(PyInt => isize);
impl_try_from_primitive!(PyInt => u8);
impl_try_from_primitive!(PyInt => u16);
impl_try_from_primitive!(PyInt => u32);
impl_try_from_primitive!(PyInt => u64);
impl_try_from_primitive!(PyInt => u128);
impl_try_from_primitive!(PyInt => usize);

// ==== List ====

impl_try_from_self_python!(PyList);

impl<P, T> PyTryFrom<Vec<P>> for Vec<T>
where
    T: PyTryFrom<P>,
{
    fn py_try_from(py: Python, item: &Vec<P>) -> PyResult<Self> {
        item.iter().map(|item| T::py_try_from(py, item)).collect()
    }
}

impl<T> PyTryFrom<Py<PyList>> for Vec<T>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, py_list: &Py<PyList>) -> PyResult<Self> {
        Self::py_try_from(py, py_list.as_ref(py))
    }
}

impl<T> PyTryFrom<PyList> for Vec<T>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, py_list: &PyList) -> PyResult<Self> {
        let mut list = Self::with_capacity(py_list.len());

        for item in py_list {
            let item = T::py_try_from(py, item)?;
            list.push(item);
        }

        Ok(list)
    }
}

impl<T> PyTryFrom<PyAny> for Vec<T>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let actual: &PyList = item.downcast()?;
        Self::py_try_from(py, actual)
    }
}

impl<T> PyTryFrom<Py<PyList>> for Box<[T]>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, py_list: &Py<PyList>) -> PyResult<Self> {
        Self::py_try_from(py, py_list.as_ref(py))
    }
}

impl<T> PyTryFrom<PyList> for Box<[T]>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, py_list: &PyList) -> PyResult<Self> {
        Vec::py_try_from(py, py_list).map(From::from)
    }
}

impl<T> PyTryFrom<PyAny> for Box<[T]>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let actual: &PyList = item.downcast()?;
        Self::py_try_from(py, actual)
    }
}

// ==== Optional[T] ====

impl<T, P> PyTryFrom<Option<P>> for Option<T>
where
    T: PyTryFrom<P>,
{
    fn py_try_from(py: Python, item: &Option<P>) -> PyResult<Self> {
        item.as_ref()
            .map_or_else(|| Ok(None), |item| T::py_try_from(py, item).map(Some))
    }
}

// ==== Set ====

impl_try_from_self_python!(PySet);

impl<T, P, Hasher> PyTryFrom<HashSet<P, Hasher>> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<P>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, set: &HashSet<P, Hasher>) -> PyResult<Self> {
        set.iter().map(|item| T::py_try_from(py, item)).collect()
    }
}

impl<T, Hasher> PyTryFrom<Py<PySet>> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, set: &Py<PySet>) -> PyResult<Self> {
        Self::py_try_from(py, set.as_ref(py))
    }
}

impl<T, Hasher> PyTryFrom<PySet> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, set: &PySet) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(set.len(), Hasher::default());
        for item in set {
            let item = T::py_try_from(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

impl<T, Hasher> PyTryFrom<PyAny> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let set: &PySet = item.downcast()?;
        Self::py_try_from(py, set)
    }
}

impl<T, P> PyTryFrom<BTreeSet<P>> for BTreeSet<T>
where
    T: Ord + PyTryFrom<P>,
{
    fn py_try_from(py: Python, set: &BTreeSet<P>) -> PyResult<Self> {
        set.iter().map(|item| T::py_try_from(py, item)).collect()
    }
}

impl<T> PyTryFrom<Py<PySet>> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, set: &Py<PySet>) -> PyResult<Self> {
        Self::py_try_from(py, set.as_ref(py))
    }
}

impl<T> PyTryFrom<PySet> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, set: &PySet) -> PyResult<Self> {
        let mut map = Self::new();
        for item in set {
            let item = T::py_try_from(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

impl<T> PyTryFrom<PyAny> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, set: &PyAny) -> PyResult<Self> {
        let set: &PySet = set.downcast()?;
        <Self as PyTryFrom<PySet>>::py_try_from(py, set)
    }
}

// ==== String ====

impl_try_from_self_python!(PyString);
impl_try_from_self_rust!(String);
impl_try_from_py_native!(PyString => String);

private_impl_py_try_from_with_pyany!(&item, _py, PyString => String {
    item.to_str().map(ToString::to_string)
});

// ==== Time ====

impl_try_from_self_python!(PyTime);

#[cfg(feature = "time")]
impl_try_from_self_rust!((Time, Option<UtcOffset>));
#[cfg(feature = "time")]
impl_try_from_py_native!(PyTime => (Time, Option<UtcOffset>));

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyTime => (Time, Option<UtcOffset>) {
    let hour: u8 = item.getattr("hour")?.downcast::<PyInt>()?.extract()?;
    let minute: u8 = item.getattr("minute")?.downcast::<PyInt>()?.extract()?;
    let seconds: u8 = item.getattr("second")?.downcast::<PyInt>()?.extract()?;
    let microseconds: u32 = item.getattr("microsecond")?.downcast::<PyInt>()?.extract()?;
    let tzinfo: Option<&PyTzInfo> = item.getattr("tzinfo")?.extract()?;
    let offset = tzinfo.map(|tzinfo| UtcOffset::py_try_from(py, tzinfo)).transpose()?;
    let timestamp = Time::from_hms_micro(hour, minute, seconds, microseconds).map_err(|err| {
        PyValueError::new_err(format!("Could not create a Rust Time from {hour}:{minute}:{seconds}.{microseconds}: {err}"))
    })?;
    Ok((timestamp, offset))
});

// ==== TzInfo ====

impl_try_from_self_python!(PyTzInfo);

#[cfg(feature = "time")]
impl_try_from_self_rust!(UtcOffset);
#[cfg(feature = "time")]
impl_try_from_py_native!(PyTzInfo => UtcOffset);

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyTzInfo => UtcOffset {
    let args: Py<PyAny> = (py.None(),).to_object(py);
    let args: &PyTuple = args.extract(py)?;
    let duration = item.call_method1("utcoffset", args).map(|any| Duration::py_try_from(py, any))??;
    let seconds = duration.whole_seconds();
    let seconds = seconds.try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot create a Rust UtcOffset from {seconds} seconds -- too many seconds!"))
    })?;
    let offset = Self::from_whole_seconds(seconds).map_err(|_| {
        PyValueError::new_err(format!("Cannot create a Rust UtcOffset from {seconds} seconds -- too many seconds!"))
    })?;
    Ok(offset)
});

// ==== IndexMap ====

#[cfg(feature = "indexmap")]
impl<K1, K2, V1, V2, S> PyTryFrom<IndexMap<K1, V1, S>> for IndexMap<K2, V2, S>
where
    K2: Eq + std::hash::Hash + PyTryFrom<K1>,
    V2: PyTryFrom<V1>,
    S: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &IndexMap<K1, V1, S>) -> PyResult<Self> {
        item.iter()
            .map(|(key, val)| {
                let key = K2::py_try_from(py, key)?;
                let val = V2::py_try_from(py, val)?;
                Ok((key, val))
            })
            .collect()
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S> PyTryFrom<PyDict> for IndexMap<K, V, S>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    S: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &PyDict) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(item.len(), S::default());
        for (key, val) in item {
            let key = K::py_try_from(py, key)?;
            let val = V::py_try_from(py, val)?;
            map.insert(key, val);
        }
        Ok(map)
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S> PyTryFrom<Py<PyDict>> for IndexMap<K, V, S>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    S: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &Py<PyDict>) -> PyResult<Self> {
        Self::py_try_from(py, item.as_ref(py))
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S> PyTryFrom<PyAny> for IndexMap<K, V, S>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    S: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyDict = item.downcast()?;
        Self::py_try_from(py, dict)
    }
}

// ============ End Implementations ==============

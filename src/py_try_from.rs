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

use pyo3::{
    types::{
        PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyFrozenSet, PyInt, PyList, PySet, PyString,
    },
    Py, PyAny, PyResult, Python,
};

#[cfg(feature = "complex")]
use num_complex::Complex;
#[cfg(feature = "complex")]
use pyo3::types::PyComplex;
#[cfg(feature = "complex")]
use std::os::raw::c_double;

#[cfg(feature = "time")]
use crate::datetime::DateTime;
#[cfg(feature = "time")]
use pyo3::{
    exceptions::PyValueError,
    types::{PyDate, PyDateTime, PyDelta, PyTime, PyTuple, PyTzInfo},
    ToPyObject,
};
#[cfg(feature = "time")]
use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

/// Convert from a Python type to a Rust type.
pub trait PyTryFrom<T>: Sized {
    /// Convert from a `Py<T>`. Defaults to delegating to `py_from_ref`.
    ///
    /// # Errors
    ///
    /// Any errors that may occur during conversion.
    fn py_try_from(py: Python, item: Py<T>) -> PyResult<Self>;

    /// Convert from a reference to the Python data.
    ///
    /// # Errors
    ///
    /// Any errors that may occur during conversion.
    fn py_try_from_ref(py: Python, item: &T) -> PyResult<Self>;
}

/// Provides a body for `py_try_from`, delegating to the implementation for the given Python type.
///
/// This should be used in other macros and for generic/container types that can't be implemented
/// entirely with a macro.
#[macro_export]
macro_rules! private_py_try_from_py_pyany_inner {
    ($item: ident, $py: ident, $py_type: ty) => {{
        let actual: $crate::pyo3::Py<$py_type> = $item.extract($py)?;
        <Self as $crate::PyTryFrom<$py_type>>::py_try_from($py, actual)
    }};
}

/// Provides a body for `py_try_from`, delegating to the related `py_try_from_ref` implementation.
///
/// This should be used in other macros and for generic/container types that can't be implemented
/// entirely with a macro.
#[macro_export]
macro_rules! private_py_try_from_py_inner {
    ($item: ident, $py: ident, $py_type: ty) => {{
        let item: &$py_type = $item.as_ref($py);
        <Self as $crate::PyTryFrom<$py_type>>::py_try_from_ref($py, item)
    }};
}

/// Generate `PyTryFrom` implementations for `PyAny` that require the `PyAny` to contain a specific Python type.
#[macro_export]
macro_rules! private_impl_py_try_from_pyany {
    ($py_type: ty => $rs_type: ty) => {
        impl $crate::PyTryFrom<$crate::pyo3::PyAny> for $rs_type {
            fn py_try_from(
                py: $crate::pyo3::Python,
                item: $crate::pyo3::Py<$crate::pyo3::PyAny>,
            ) -> $crate::pyo3::PyResult<Self> {
                $crate::private_py_try_from_py_pyany_inner!(item, py, $py_type)
            }

            fn py_try_from_ref(
                py: $crate::pyo3::Python,
                item: &$crate::pyo3::PyAny,
            ) -> $crate::pyo3::PyResult<Self> {
                let actual: $crate::pyo3::Py<$py_type> = item.extract()?;
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
        impl PyTryFrom<$py_type> for $rs_type {
            fn py_try_from(
                py: $crate::pyo3::Python,
                item: $crate::pyo3::Py<$py_type>,
            ) -> $crate::pyo3::PyResult<Self> {
                $crate::private_py_try_from_py_inner!(item, py, $py_type)
            }
            fn py_try_from_ref(
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

/// Implements [`PyTryFrom`] for primitive types by just calling `extract()`.
macro_rules! impl_try_from_primitive {
    ($py_type: ty => $rs_type: ty) => {
        private_impl_py_try_from_with_pyany!(&item, _py, $py_type => $rs_type { item.extract() });
    };
}

// ============ Begin Implementations ==============

// ==== Bool ====

private_impl_py_try_from_with_pyany!(&item, _py, PyBool => bool {
    Ok(item.is_true())
});

// ==== ByteArray ====

private_impl_py_try_from!(&item, _py, PyByteArray => Vec<u8> {
    Ok(item.to_vec())
});

// ==== Bytes ====

private_impl_py_try_from!(&item, _py, PyBytes => Vec<u8> {
    Ok(item.as_bytes().to_vec())
});

// ==== Complex ====

#[cfg(feature = "complex")]
impl<F> PyTryFrom<PyComplex> for Complex<F>
where
    F: Copy + From<c_double>,
{
    fn py_try_from(py: Python, item: Py<PyComplex>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PyComplex)
    }
    fn py_try_from_ref(_py: Python, item: &PyComplex) -> PyResult<Self> {
        Ok(Self {
            re: F::from(item.real()),
            im: F::from(item.imag()),
        })
    }
}

#[cfg(feature = "complex")]
impl<F> PyTryFrom<PyAny> for Complex<F>
where
    F: Copy + From<c_double>,
{
    fn py_try_from(py: Python, item: Py<PyAny>) -> PyResult<Self> {
        private_py_try_from_py_pyany_inner!(item, py, PyComplex)
    }
    fn py_try_from_ref(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyComplex = item.downcast()?;
        Self::py_try_from_ref(py, dict)
    }
}

// ==== Date ====

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyDate => Date {
    let year = item.getattr("year").map(|any| i32::py_try_from_ref(py, any))??;
    let month: u8 = item.getattr("month").map(|any| u8::py_try_from_ref(py, any))??; // 1-12
    let month: Month = month.try_into()
        .map_err(|_| {
            PyValueError::new_err(format!("Expected date month to be within 0-12, got {month}"))
        })?;
    let day = item.getattr("day").map(|any| u8::py_try_from_ref(py, any))??; // 1-X

    Self::from_calendar_date(year, month, day).map_err(|err| {
        PyValueError::new_err(format!("Failed to create Date object: {err}"))
    })
});

// ==== DateTime ====

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyDateTime => DateTime {
    let date = item.call_method0("date").map(|date| Date::py_try_from_ref(py, date))??;
    let (time, offset) = item.call_method0("timetz")
        .map(|time| <(Time, Option<UtcOffset>)>::py_try_from_ref(py, time))??;
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

// ==== Dict ====

impl<K, V, Hasher> PyTryFrom<PyDict> for HashMap<K, V, Hasher>
where
    K: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: Py<PyDict>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PyDict)
    }
    fn py_try_from_ref(py: Python, item: &PyDict) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(item.len(), Hasher::default());
        for (key, val) in item.iter() {
            let key = K::py_try_from_ref(py, key)?;
            let val = V::py_try_from_ref(py, val)?;
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
    fn py_try_from(py: Python, item: Py<PyAny>) -> PyResult<Self> {
        private_py_try_from_py_pyany_inner!(item, py, PyDict)
    }
    fn py_try_from_ref(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyDict = item.downcast()?;
        Self::py_try_from_ref(py, dict)
    }
}

impl<K, V> PyTryFrom<PyDict> for BTreeMap<K, V>
where
    K: Ord + PyTryFrom<PyAny>,
    V: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: Py<PyDict>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PyDict)
    }
    fn py_try_from_ref(py: Python, item: &PyDict) -> PyResult<Self> {
        let mut map = Self::new();
        for (key, val) in item.iter() {
            let key = K::py_try_from_ref(py, key)?;
            let val = V::py_try_from_ref(py, val)?;
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
    fn py_try_from(py: Python, item: Py<PyAny>) -> PyResult<Self> {
        private_py_try_from_py_pyany_inner!(item, py, PyDict)
    }
    fn py_try_from_ref(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyDict = item.downcast()?;
        <Self as PyTryFrom<PyDict>>::py_try_from_ref(py, dict)
    }
}

// ==== Float ====

impl_try_from_primitive!(PyFloat => f32);
impl_try_from_primitive!(PyFloat => f64);

// ==== FrozenSet ====

impl<T, Hasher> PyTryFrom<PyFrozenSet> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: Py<PyFrozenSet>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PyFrozenSet)
    }
    fn py_try_from_ref(py: Python, set: &PyFrozenSet) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(set.len(), Hasher::default());
        for item in set.iter() {
            let item = T::py_try_from_ref(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

impl<T> PyTryFrom<PyFrozenSet> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: Py<PyFrozenSet>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PyFrozenSet)
    }
    fn py_try_from_ref(py: Python, set: &PyFrozenSet) -> PyResult<Self> {
        let mut map = Self::new();
        for item in set.iter() {
            let item = T::py_try_from_ref(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

// ==== Integer ====

impl_try_from_primitive!(PyInt => i8);
impl_try_from_primitive!(PyInt => i16);
impl_try_from_primitive!(PyInt => i32);
impl_try_from_primitive!(PyInt => i64);
impl_try_from_primitive!(PyInt => i128);
impl_try_from_primitive!(PyInt => u8);
impl_try_from_primitive!(PyInt => u16);
impl_try_from_primitive!(PyInt => u32);
impl_try_from_primitive!(PyInt => u64);
impl_try_from_primitive!(PyInt => u128);

// ==== List ====

impl<T> PyTryFrom<PyList> for Vec<T>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: Py<PyList>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PyList)
    }

    fn py_try_from_ref(py: Python, py_list: &PyList) -> PyResult<Self> {
        let mut list = Self::with_capacity(py_list.len());

        for item in py_list.iter() {
            let item = T::py_try_from_ref(py, item)?;
            list.push(item);
        }

        Ok(list)
    }
}

impl<T> PyTryFrom<PyAny> for Vec<T>
where
    T: PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: Py<PyAny>) -> PyResult<Self> {
        private_py_try_from_py_pyany_inner!(item, py, PyList)
    }

    fn py_try_from_ref(py: Python, item: &PyAny) -> PyResult<Self> {
        let actual: &PyList = item.downcast()?;
        Self::py_try_from_ref(py, actual)
    }
}

// ==== Set ====

impl<T, Hasher> PyTryFrom<PySet> for HashSet<T, Hasher>
where
    T: Eq + std::hash::Hash + PyTryFrom<PyAny>,
    Hasher: std::hash::BuildHasher + Default,
{
    fn py_try_from(py: Python, item: Py<PySet>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PySet)
    }
    fn py_try_from_ref(py: Python, set: &PySet) -> PyResult<Self> {
        let mut map = Self::with_capacity_and_hasher(set.len(), Hasher::default());
        for item in set.iter() {
            let item = T::py_try_from_ref(py, item)?;
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
    fn py_try_from(py: Python, item: Py<PyAny>) -> PyResult<Self> {
        private_py_try_from_py_pyany_inner!(item, py, PySet)
    }
    fn py_try_from_ref(py: Python, item: &PyAny) -> PyResult<Self> {
        let set: &PySet = item.downcast()?;
        Self::py_try_from_ref(py, set)
    }
}

impl<T> PyTryFrom<PySet> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: Py<PySet>) -> PyResult<Self> {
        private_py_try_from_py_inner!(item, py, PySet)
    }
    fn py_try_from_ref(py: Python, set: &PySet) -> PyResult<Self> {
        let mut map = Self::new();
        for item in set.iter() {
            let item = T::py_try_from_ref(py, item)?;
            map.insert(item);
        }
        Ok(map)
    }
}

impl<T> PyTryFrom<PyAny> for BTreeSet<T>
where
    T: Ord + PyTryFrom<PyAny>,
{
    fn py_try_from(py: Python, item: Py<PyAny>) -> PyResult<Self> {
        private_py_try_from_py_pyany_inner!(item, py, PySet)
    }
    fn py_try_from_ref(py: Python, set: &PyAny) -> PyResult<Self> {
        let set: &PySet = set.downcast()?;
        <Self as PyTryFrom<PySet>>::py_try_from_ref(py, set)
    }
}

// ==== String ====

private_impl_py_try_from_with_pyany!(&item, _py, PyString => String {
    item.to_str().map(ToString::to_string)
});

// ==== Time ====

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyTime => (Time, Option<UtcOffset>) {
    let hour: u8 = item.getattr("hour")?.downcast::<PyInt>()?.extract()?;
    let minute: u8 = item.getattr("minute")?.downcast::<PyInt>()?.extract()?;
    let seconds: u8 = item.getattr("second")?.downcast::<PyInt>()?.extract()?;
    let microseconds: u32 = item.getattr("microsecond")?.downcast::<PyInt>()?.extract()?;
    let tzinfo: Option<&PyTzInfo> = item.getattr("tzinfo")?.extract()?;
    let offset = tzinfo.map(|tzinfo| UtcOffset::py_try_from_ref(py, tzinfo)).transpose()?;
    let timestamp = Time::from_hms_micro(hour, minute, seconds, microseconds).map_err(|err| {
        PyValueError::new_err(format!("Could not create a Rust Time from {hour}:{minute}:{seconds}.{microseconds}: {err}"))
    })?;
    Ok((timestamp, offset))
});

// ==== TzInfo ====

#[cfg(feature = "time")]
private_impl_py_try_from_with_pyany!(&item, py, PyTzInfo => UtcOffset {
    let args: Py<PyAny> = (py.None(),).to_object(py);
    let args: &PyTuple = args.extract(py)?;
    let duration = item.call_method1("utcoffset", args).map(|any| Duration::py_try_from_ref(py, any))??;
    let seconds = duration.whole_seconds();
    let seconds = seconds.try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot create a Rust UtcOffset from {seconds} seconds -- too many seconds!"))
    })?;
    let offset = Self::from_whole_seconds(seconds).map_err(|_| {
        PyValueError::new_err(format!("Cannot create a Rust UtcOffset from {seconds} seconds -- too many seconds!"))
    })?;
    Ok(offset)
});

// ============ End Implementations ==============

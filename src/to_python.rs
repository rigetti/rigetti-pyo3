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

//! Unifying conversion traits from Rust data to Python.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use pyo3::conversion::IntoPy;
use pyo3::types::{
    PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyFrozenSet, PyList, PyLong, PySet, PyString,
};
use pyo3::{Py, PyAny, PyResult, Python, ToPyObject};

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
};
#[cfg(feature = "time")]
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

/// Convert from a Rust type into a Python type.
pub trait ToPython<T: ToPyObject> {
    /// Convert from Rust `self` into a Python type.
    ///
    /// # Errors
    ///
    /// Any failure while converting to Python.
    fn to_python(&self, py: Python) -> PyResult<Py<T>>;
}

// Simple blanket impl based on pyo3::IntoPy does not work because, e.g., `String`
// does not impl `IntoPy<Py<PyString>>`, only `IntoPy<Py<PyAny>>`.
//
// Using the `impl IntoPy<Py<PyAny>>` is not viable either, because that is liable to
// errors (panics, because this trait doesn't return `Result`). We'd have to clarify
// which `std` types can convert to which `pyo3` types, and at that point we might as
// well implement conversion without using `expect()`.

/// Implement [`ToPython<PyAny>`] for the given Rust type by delegating to its implementation for
/// the given Python type. Arguments are the same as for
/// [`private_impl_to_python_for`](crate::private_impl_to_python_for).
#[macro_export]
macro_rules! private_impl_to_python_to_pyany {
    (&$($lt: lifetime)? $self: ident, $py: ident, $rs_type: ty => $py_type: ty) => {
        $crate::private_impl_to_python_for!(&$($lt)? $self, $py, $rs_type => $crate::pyo3::PyAny {
            <Self as $crate::ToPython<$py_type>>::to_python($self, $py).map(|item| item.into_py($py))
        });
    }
}

/// Implement [`ToPython`] once for the given Rust type. Will implement for a reference to the type
/// if a lifetime is provided.
#[macro_export]
macro_rules! private_impl_to_python_for {
    (&$($lt: lifetime)? $self: ident, $py: ident, $rs_type: ty => $py_type: ty $convert: block) => {
        impl$(<$lt>)? $crate::ToPython<$py_type> for $(&$lt)? $rs_type {
            fn to_python(&$self, $py: $crate::pyo3::Python<'_>) -> $crate::pyo3::PyResult<$crate::pyo3::Py<$py_type>> {
                $convert
            }
        }
    }
}

/// Implement [`ToPython`] on the given Rust type and a reference to it.
#[macro_export]
macro_rules! private_impl_to_python_with_reference {
    (&$self: ident, $py: ident, $rs_type: ty => $py_type: ty $convert: block) => {
        $crate::private_impl_to_python_for!(&$self, $py, $rs_type => $py_type $convert);
        $crate::private_impl_to_python_for!(&'a $self, $py, $rs_type => $py_type {
            <$rs_type as $crate::ToPython<$py_type>>::to_python(*$self, $py)
        });
    };
}

/// Implement [`ToPython`] multiple times for the given types, accounting for owned/reference and [`PyAny`](crate::pyo3::PyAny).
#[macro_export]
macro_rules! private_impl_to_python_with_reference_and_pyany {
    (&$self: ident, $py: ident, $rs_type: ty => $py_type: ty $convert: block) => {
        $crate::private_impl_to_python_with_reference!(&$self, $py, $rs_type => $py_type $convert);
        $crate::private_impl_to_python_to_pyany!(&$self, $py, $rs_type => $py_type);
        $crate::private_impl_to_python_to_pyany!(&'a $self, $py, $rs_type => $py_type);
    };
}

/// Implements [`IntoPython`] by converting to `Py<PyAny>` and extracting `Py<T>` from that.
///
/// For types like integers, this is only way to convert.
macro_rules! impl_for_primitive {
    ($rs_type: ty => $py_type: ty) => {
        private_impl_to_python_with_reference_and_pyany!(&self, py, $rs_type => $py_type {
            // No way to convert except via ToPyObject and downcasting.
            self.into_py(py).extract(py)
        });
    };
}

// ============ Begin Implementations ==============

// ==== Bool ====

private_impl_to_python_with_reference_and_pyany!(&self, py, bool => PyBool {
    Ok(PyBool::new(py, *self).into_py(py))
});

// ==== ByteArray ====

private_impl_to_python_for!(&'a self, py, [u8] => PyByteArray {
    Ok(PyByteArray::new(py, self).into_py(py))
});

private_impl_to_python_with_reference!(&self, py, Vec<u8> => PyByteArray {
    self.as_slice().to_python(py)
});

// ==== Bytes ====

private_impl_to_python_for!(&'a self, py, [u8] => PyBytes {
    Ok(PyBytes::new(py, self).into_py(py))
});

private_impl_to_python_with_reference!(&self, py, Vec<u8> => PyBytes {
    self.as_slice().to_python(py)
});

// ==== Complex ====

#[cfg(feature = "complex")]
impl<'a, F> ToPython<PyComplex> for &'a Complex<F>
where
    F: Copy + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyComplex>> {
        Ok(PyComplex::from_complex(py, **self).into_py(py))
    }
}

#[cfg(feature = "complex")]
impl<F> ToPython<PyComplex> for Complex<F>
where
    F: Copy + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyComplex>> {
        <&Self as ToPython<PyComplex>>::to_python(&self, py)
    }
}

#[cfg(feature = "complex")]
impl<'a, F> ToPython<PyAny> for &'a Complex<F>
where
    F: Copy + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyComplex>>::to_python(self, py).map(|c| c.into_py(py))
    }
}

#[cfg(feature = "complex")]
impl<F> ToPython<PyAny> for Complex<F>
where
    F: Copy + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyComplex>>::to_python(self, py).map(|c| c.into_py(py))
    }
}

// ==== Date ====

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, Date => PyDate {
    let year: i32 = self.year();
    let month: u8 = self.month().into();
    let day: u8 = self.day();
    PyDate::new(py, year, month, day).map(|date| date.into_py(py))
});

// ==== DateTime ====

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, DateTime => PyDateTime {
    match self {
        Self::Primitive(datetime) => datetime.to_python(py),
        Self::Offset(datetime) => datetime.to_python(py),
    }
});

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, PrimitiveDateTime => PyDateTime {
    let date = self.date();
    let time = self.time();
    let year: i32 = date.year();
    let month: u8 = date.month().into();
    let day: u8 = date.day();
    let hour = time.hour();
    let minute = time.minute();
    let second = time.second();
    let microsecond = time.microsecond();
    PyDateTime::new(py, year, month, day, hour, minute, second, microsecond, None).map(|dt| dt.into_py(py))
});

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, OffsetDateTime => PyDateTime {
    let date = self.date();
    let time = self.time();
    let offset = self.offset();
    let year: i32 = date.year();
    let month: u8 = date.month().into();
    let day: u8 = date.day();
    let hour = time.hour();
    let minute = time.minute();
    let second = time.second();
    let microsecond = time.microsecond();
    let tzinfo: Py<PyTzInfo> = offset.to_python(py)?;
    let tzinfo = tzinfo.as_ref(py);
    PyDateTime::new(py, year, month, day, hour, minute, second, microsecond, Some(tzinfo)).map(|dt| dt.into_py(py))
});

// ==== Delta ====

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, Duration => PyDelta {
    let days: i32 = self.whole_days().try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot fit {} days into a 32-bit signed integer", self.whole_days()))
    })?;
    let seconds: i32 = self.whole_seconds().try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot fit {} seconds into a 32-bit signed integer", self.whole_seconds()))
    })?;
    let microseconds:i32 = self.whole_microseconds().try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot fit {} microseconds into a 32-bit signed integer", self.whole_microseconds()))
    })?;
    PyDelta::new(py, days, seconds, microseconds, true).map(|delta| delta.into_py(py))
});

// ==== Dict ====

impl<'a, K, V, Hasher> ToPython<PyDict> for &'a HashMap<K, V, Hasher>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (key, val) in *self {
            let pykey = key.to_python(py)?;
            let pyval = val.to_python(py)?;
            dict.set_item(pykey, pyval)?;
        }
        Ok(dict.into_py(py))
    }
}

impl<K, V, Hasher> ToPython<PyDict> for HashMap<K, V, Hasher>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        <&Self as ToPython<PyDict>>::to_python(&self, py)
    }
}

impl<K, V, Hasher> ToPython<PyAny> for HashMap<K, V, Hasher>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyDict>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<'a, K, V, Hasher> ToPython<PyAny> for &'a HashMap<K, V, Hasher>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyDict>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<'a, K, V> ToPython<PyDict> for &'a BTreeMap<K, V>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (key, val) in *self {
            let pykey = key.to_python(py)?;
            let pyval = val.to_python(py)?;
            dict.set_item(pykey, pyval)?;
        }
        Ok(dict.into_py(py))
    }
}

impl<K, V> ToPython<PyDict> for BTreeMap<K, V>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        <&Self as ToPython<PyDict>>::to_python(&self, py)
    }
}

impl<'a, K, V> ToPython<PyAny> for &'a BTreeMap<K, V>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyDict>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<K, V> ToPython<PyAny> for BTreeMap<K, V>
where
    K: ToPython<PyAny> + std::fmt::Debug,
    V: ToPython<PyAny>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyDict>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

// ==== Float ====

impl_for_primitive!(f32 => PyFloat);
impl_for_primitive!(f64 => PyFloat);

// ==== FrozenSet ====

impl<'a, T, Hasher> ToPython<PyFrozenSet> for &'a HashSet<T, Hasher>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        let elements = self
            .iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()?;
        PyFrozenSet::new(py, &elements).map(|set| set.into_py(py))
    }
}

impl<T, Hasher> ToPython<PyFrozenSet> for HashSet<T, Hasher>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        <&Self as ToPython<PyFrozenSet>>::to_python(&self, py)
    }
}

impl<'a, T> ToPython<PyFrozenSet> for &'a BTreeSet<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        let elements = self
            .iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()?;
        PyFrozenSet::new(py, &elements).map(|set| set.into_py(py))
    }
}

impl<T> ToPython<PyFrozenSet> for BTreeSet<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        <&Self as ToPython<PyFrozenSet>>::to_python(&self, py)
    }
}

// ==== Integer ====

impl_for_primitive!(i8 => PyLong);
impl_for_primitive!(i16 => PyLong);
impl_for_primitive!(i32 => PyLong);
impl_for_primitive!(i64 => PyLong);
impl_for_primitive!(i128 => PyLong);
impl_for_primitive!(u8 => PyLong);
impl_for_primitive!(u16 => PyLong);
impl_for_primitive!(u32 => PyLong);
impl_for_primitive!(u64 => PyLong);
impl_for_primitive!(u128 => PyLong);

// ==== List ====

impl<'a, T> ToPython<PyList> for &'a [T]
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        let elements = self
            .iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyList::new(py, elements).into_py(py))
    }
}

impl<'a, T> ToPython<PyAny> for &'a [T]
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PyList>>::to_python(self, py).map(|list| list.into_py(py))
    }
}

impl<T> ToPython<PyList> for Vec<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        self.as_slice().to_python(py)
    }
}

impl<'a, T> ToPython<PyList> for &'a Vec<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        self.as_slice().to_python(py)
    }
}

impl<T> ToPython<PyAny> for Vec<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.as_slice().to_python(py)
    }
}

impl<'a, T> ToPython<PyAny> for &'a Vec<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.as_slice().to_python(py)
    }
}

// ==== Set ====

impl<'a, T, Hasher> ToPython<PySet> for &'a HashSet<T, Hasher>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        // Using PySet::new seems to do extra cloning, so build manually.
        let set = PySet::empty(py)?;
        for item in self.iter() {
            set.add(item.to_python(py)?)?;
        }
        Ok(set.into_py(py))
    }
}

impl<T, Hasher> ToPython<PySet> for HashSet<T, Hasher>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        <&Self as ToPython<PySet>>::to_python(&self, py)
    }
}

impl<'a, T, Hasher> ToPython<PyAny> for &'a HashSet<T, Hasher>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PySet>>::to_python(self, py).map(|set| set.into_py(py))
    }
}

impl<T, Hasher> ToPython<PyAny> for HashSet<T, Hasher>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PySet>>::to_python(self, py).map(|set| set.into_py(py))
    }
}

impl<'a, T> ToPython<PySet> for &'a BTreeSet<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        // Using PySet::new seems to do extra cloning, so build manually.
        let set = PySet::empty(py)?;
        for item in self.iter() {
            set.add(item.to_python(py)?)?;
        }
        Ok(set.into_py(py))
    }
}

impl<T> ToPython<PySet> for BTreeSet<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        <&Self as ToPython<PySet>>::to_python(&self, py)
    }
}

impl<'a, T> ToPython<PyAny> for &'a BTreeSet<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PySet>>::to_python(self, py).map(|set| set.into_py(py))
    }
}

impl<T> ToPython<PyAny> for BTreeSet<T>
where
    T: ToPython<PyAny> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<PySet>>::to_python(self, py).map(|set| set.into_py(py))
    }
}

// ==== String ====

private_impl_to_python_for!(&'a self, py, str => PyString {
    Ok(PyString::new(py, self).into_py(py))
});
private_impl_to_python_to_pyany!(&'a self, py, str => PyString);

private_impl_to_python_with_reference_and_pyany!(&self, py, String => PyString {
    self.as_str().to_python(py)
});

// ==== Time ====

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, (Time, Option<UtcOffset>) => PyTime {
    let (time, offset) = self;
    let hour = time.hour();
    let minute = time.minute();
    let second = time.second();
    let microsecond = time.microsecond();
    let tzinfo: Option<Py<PyTzInfo>> = offset.map(|offset| {
        offset.to_python(py)
    }).transpose()?;
    let tzinfo = tzinfo.as_ref().map(|tzinfo| tzinfo.as_ref(py));
    PyTime::new(py, hour, minute, second, microsecond, tzinfo).map(|time| time.into_py(py))
});

// ==== TzInfo ====

#[cfg(feature = "time")]
private_impl_to_python_with_reference_and_pyany!(&self, py, UtcOffset => PyTzInfo {
    let datetime = py.import("datetime")?;
    let timezone = datetime.getattr("timezone")?;
    let (hour, minute, second) = self.as_hms();
    let seconds = i64::from(second) + 60 * (i64::from(minute) + (60 * i64::from(hour)));
    let duration = Duration::new(seconds, 0);
    let delta: Py<PyDelta> = duration.to_python(py)?;
    let args = (delta,).to_object(py);
    let args: &PyTuple = args.extract(py)?;
    let tzinfo = timezone.call1(args)?;
    tzinfo.extract()
});

// ============ End Implementations ==============

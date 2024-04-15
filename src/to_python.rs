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
use std::hash::BuildHasher;

use pyo3::conversion::IntoPy;
use pyo3::types::PyComplex;
use pyo3::types::{
    PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyFrozenSet, PyList, PyLong, PySet, PyString,
};
use pyo3::{
    exceptions::PyValueError,
    types::{PyDate, PyDateTime, PyDelta, PyTime, PyTzInfo},
};
use pyo3::{Py, PyAny, PyResult, Python, ToPyObject};

#[cfg(feature = "complex")]
use num_complex::Complex;
#[cfg(feature = "complex")]
use num_traits::{Float, FloatConst};
#[cfg(feature = "complex")]
use std::os::raw::c_double;

#[cfg(feature = "time")]
use crate::datetime::DateTime;
#[cfg(feature = "time")]
use pyo3::types::PyTuple;
#[cfg(feature = "time")]
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

#[cfg(feature = "indexmap")]
use indexmap::IndexMap;

/// Convert from a Rust type into a Python type.
pub trait ToPython<P: ToPyObject> {
    /// Convert from Rust `self` into a Python type.
    ///
    /// # Errors
    ///
    /// Any failure while converting to Python.
    fn to_python(&self, py: Python) -> PyResult<P>;
}

impl<'a, T, P> ToPython<P> for &'a Box<T>
where
    T: ToPython<P>,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<P> {
        T::to_python(self, py)
    }
}

impl<T, P> ToPython<P> for Box<T>
where
    T: ToPython<P>,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<P> {
        T::to_python(self, py)
    }
}

/// Provides a generic implementation of [`ToPython`] for heterogenous tuples of types that themselves implement
/// [`ToPython`].
macro_rules! impl_to_python_for_tuple {
    ($($idx:tt $t:tt $p:tt),+) => {
        impl<$($t,)+ $($p,)+> ToPython<($($p,)+)> for ($($t,)+)
        where
            $($t: ToPython<$p>, $p: ToPyObject,)+

        {
            fn to_python(&self, py: Python) -> PyResult<($($p,)+)> {
                Ok(($(
                    $t :: to_python(&self.$idx, py)?,
                )+))
            }
        }
    };
}

// Implement [`ToPython`] for tuples of length 1 to 12, 12 being the maximum arity that [`pyo3::ToPyObject`]
// is implemented for.
impl_to_python_for_tuple!(0 T0 P0);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8, 9 T9 P9);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8, 9 T9 P9, 10 T10 P10);
impl_to_python_for_tuple!(0 T0 P0, 1 T1 P1, 2 T2 P2, 3 T3 P3, 4 T4 P4, 5 T5 P5, 6 T6 P6, 7 T7 P7, 8 T8 P8, 9 T9 P9, 10 T10 P10, 11 T11 P11);

/// Implement [`ToPython`] once for the given Rust type. Will implement for a reference to the type
/// if a lifetime is provided.
#[macro_export]
macro_rules! private_impl_to_python_for {
    (&$($lt: lifetime)? $self: ident, $py: ident, $rs_type: ty => $py_type: ty $convert: block) => {
        #[allow(clippy::use_self)]
        impl$(<$lt>)? $crate::ToPython<$py_type> for $(&$lt)? $rs_type {
            fn to_python(&$self, $py: $crate::pyo3::Python<'_>) -> $crate::pyo3::PyResult<$py_type> {
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

/// Implement [`ToPython<Py<PyAny>>`] for a type using its implementation for `ToPython<P>` where `P: ToPyObject`.
#[macro_export]
macro_rules! private_impl_to_python_pyany {
    ($rs_type: ty => $py_type: ty) => {
        private_impl_to_python_with_reference!(&self, py, $rs_type => $crate::pyo3::Py<$crate::pyo3::PyAny> {
            $crate::ToPython::<$py_type>::to_python(self, py).map(|item| $crate::pyo3::ToPyObject::to_object(&item, py))
        });
    }
}

/// Implements [`IntoPython`] by converting to `Py<PyAny>` and extracting `Py<T>` from that.
///
/// For types like integers, this is only way to convert.
macro_rules! impl_for_primitive {
    ($rs_type: ty => $py_type: ty) => {
        private_impl_to_python_with_reference!(&self, py, $rs_type => $py_type {
            // No way to convert except via ToPyObject and downcasting.
            self.into_py(py).extract(py)
        });
    };
}

/// Implement `ToPython<Self>` for a given type.
macro_rules! impl_for_self {
    ($type: ty) => {
        private_impl_to_python_with_reference!(&self, _py, $type => $type {
            Ok(self.clone())
        });
        private_impl_to_python_pyany!($type => $type);
    }
}

// ============ Begin Implementations ==============

// ==== Bool ====

impl_for_self!(bool);
impl_for_self!(Py<PyBool>);

private_impl_to_python_with_reference!(&self, py, bool => Py<PyBool> {
    Ok(PyBool::new(py, *self).into_py(py))
});

// ==== ByteArray ====

impl_for_self!(Py<PyByteArray>);

private_impl_to_python_with_reference!(&self, py, [u8] => Py<PyByteArray> {
    Ok(PyByteArray::new(py, self).into_py(py))
});

private_impl_to_python_with_reference!(&self, py, Vec<u8> => Py<PyByteArray> {
    self.as_slice().to_python(py)
});

// ==== Bytes ====

impl_for_self!(Py<PyBytes>);

private_impl_to_python_with_reference!(&self, py, [u8] => Py<PyBytes> {
    Ok(PyBytes::new(py, self).into_py(py))
});

private_impl_to_python_with_reference!(&self, py, Vec<u8> => Py<PyBytes> {
    self.as_slice().to_python(py)
});

// ==== Complex ====

impl_for_self!(Py<PyComplex>);

#[cfg(feature = "complex")]
impl<'a, F> ToPython<Py<PyComplex>> for &'a Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyComplex>> {
        Ok(PyComplex::from_complex(py, **self).into_py(py))
    }
}

#[cfg(feature = "complex")]
impl<F> ToPython<Py<PyComplex>> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyComplex>> {
        <&Self as ToPython<Py<PyComplex>>>::to_python(&self, py)
    }
}

#[cfg(feature = "complex")]
impl<'a, F> ToPython<Py<PyAny>> for &'a Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        Ok(PyComplex::from_complex(py, **self).into_py(py))
    }
}

#[cfg(feature = "complex")]
impl<F> ToPython<Py<PyAny>> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <&Self as ToPython<Py<PyAny>>>::to_python(&self, py)
    }
}

// ==== Date ====

impl_for_self!(Py<PyDate>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, Date => Py<PyDate> {
    let year: i32 = self.year();
    let month: u8 = self.month().into();
    let day: u8 = self.day();
    PyDate::new(py, year, month, day).map(|date| date.into_py(py))
});

#[cfg(feature = "time")]
private_impl_to_python_pyany!(Date => Py<PyDate>);

// ==== DateTime ====

impl_for_self!(Py<PyDateTime>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, DateTime => Py<PyDateTime> {
    match self {
        Self::Primitive(datetime) => datetime.to_python(py),
        Self::Offset(datetime) => datetime.to_python(py),
    }
});

#[cfg(feature = "time")]
private_impl_to_python_pyany!(DateTime => Py<PyDateTime>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, PrimitiveDateTime => Py<PyDateTime> {
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
private_impl_to_python_pyany!(PrimitiveDateTime => Py<PyDateTime>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, OffsetDateTime => Py<PyDateTime> {
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

#[cfg(feature = "time")]
private_impl_to_python_pyany!(OffsetDateTime => Py<PyDateTime>);

// ==== Delta ====

impl_for_self!(Py<PyDelta>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, Duration => Py<PyDelta> {
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

#[cfg(feature = "time")]
private_impl_to_python_pyany!(Duration => Py<PyDelta>);

private_impl_to_python_with_reference!(&self, py, std::time::Duration => Py<PyDelta> {
    /// The number of seconds in a day.
    const DAY_FACTOR: u64 = 60 * 60 * 24;
    let microseconds = self.as_micros() % 1_000_000;
    let seconds = self.as_secs() % DAY_FACTOR;
    let days = self.as_secs() / DAY_FACTOR;

    let microseconds: i32 = microseconds.try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot fit {microseconds} microseconds into a 32-bit signed integer"))
    })?;

    let seconds: i32 = seconds.try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot fit {seconds} seconds into a 32-bit signed integer"))
    })?;

    let days: i32 = days.try_into().map_err(|_| {
        PyValueError::new_err(format!("Cannot fit {days} days into a 32-bit signed integer"))
    })?;

    PyDelta::new(py, days, seconds, microseconds, true).map(|delta| delta.into_py(py))
});

private_impl_to_python_pyany!(std::time::Duration => Py<PyDelta>);

// ==== Dict ====

impl_for_self!(Py<PyDict>);

impl<'a, K1, K2, V1, V2, Hasher> ToPython<HashMap<K2, V2>> for &'a HashMap<K1, V1, Hasher>
where
    K1: ToPython<K2>,
    V1: ToPython<V2>,
    K2: ToPyObject + Eq + std::hash::Hash,
    V2: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<HashMap<K2, V2>> {
        self.iter()
            .map(|(key, val)| {
                let key = key.to_python(py)?;
                let val = val.to_python(py)?;
                Ok((key, val))
            })
            .collect::<Result<_, _>>()
    }
}

impl<'a, K, V, Hasher> ToPython<Py<PyDict>> for &'a HashMap<K, V, Hasher>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
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

impl<'a, K, V, Hasher> ToPython<Py<PyAny>> for &'a HashMap<K, V, Hasher>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyDict>>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<K1, K2, V1, V2, Hasher> ToPython<HashMap<K2, V2>> for HashMap<K1, V1, Hasher>
where
    K1: ToPython<K2>,
    V1: ToPython<V2>,
    K2: ToPyObject + Eq + std::hash::Hash,
    V2: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<HashMap<K2, V2>> {
        <&Self as ToPython<HashMap<K2, V2>>>::to_python(&self, py)
    }
}

impl<K, V, Hasher> ToPython<Py<PyDict>> for HashMap<K, V, Hasher>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        <&Self as ToPython<Py<PyDict>>>::to_python(&self, py)
    }
}

impl<K, V, Hasher> ToPython<Py<PyAny>> for HashMap<K, V, Hasher>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyDict>>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<'a, K1, K2, V1, V2> ToPython<BTreeMap<K2, V2>> for &'a BTreeMap<K1, V1>
where
    K1: ToPython<K2> + std::fmt::Debug,
    V1: ToPython<V2>,
    K2: ToPyObject + Ord,
    V2: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<BTreeMap<K2, V2>> {
        let mut map = BTreeMap::new();
        for (key, val) in *self {
            let pykey = key.to_python(py)?;
            let pyval = val.to_python(py)?;
            map.insert(pykey, pyval);
        }
        Ok(map)
    }
}

impl<'a, K, V> ToPython<Py<PyDict>> for &'a BTreeMap<K, V>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
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

impl<'a, K, V> ToPython<Py<PyAny>> for &'a BTreeMap<K, V>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyDict>>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<K1, K2, V1, V2> ToPython<BTreeMap<K2, V2>> for BTreeMap<K1, V1>
where
    K1: ToPython<K2> + std::fmt::Debug,
    V1: ToPython<V2>,
    K2: ToPyObject + Ord,
    V2: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<BTreeMap<K2, V2>> {
        <&Self as ToPython<BTreeMap<K2, V2>>>::to_python(&self, py)
    }
}

impl<K, V> ToPython<Py<PyDict>> for BTreeMap<K, V>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        <&Self as ToPython<Py<PyDict>>>::to_python(&self, py)
    }
}

impl<K, V> ToPython<Py<PyAny>> for BTreeMap<K, V>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyDict>>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

// ==== Float ====

impl_for_self!(Py<PyFloat>);
impl_for_self!(f32);
impl_for_self!(f64);

impl_for_primitive!(f32 => Py<PyFloat>);
impl_for_primitive!(f64 => Py<PyFloat>);

// ==== FrozenSet ====

impl<'a, T, Hasher> ToPython<Py<PyFrozenSet>> for &'a HashSet<T, Hasher>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        let elements = self
            .iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()?;
        PyFrozenSet::new(py, &elements).map(|set| set.into_py(py))
    }
}

impl<T, Hasher> ToPython<Py<PyFrozenSet>> for HashSet<T, Hasher>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        <&Self as ToPython<Py<PyFrozenSet>>>::to_python(&self, py)
    }
}

impl<'a, T> ToPython<Py<PyFrozenSet>> for &'a BTreeSet<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        let elements = self
            .iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()?;
        PyFrozenSet::new(py, &elements).map(|set| set.into_py(py))
    }
}

impl<T> ToPython<Py<PyFrozenSet>> for BTreeSet<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyFrozenSet>> {
        <&Self as ToPython<Py<PyFrozenSet>>>::to_python(&self, py)
    }
}

// ==== Integer ====

impl_for_self!(Py<PyLong>);
impl_for_self!(i8);
impl_for_self!(i16);
impl_for_self!(i32);
impl_for_self!(i64);
impl_for_self!(i128);
impl_for_self!(isize);
impl_for_self!(u8);
impl_for_self!(u16);
impl_for_self!(u32);
impl_for_self!(u64);
impl_for_self!(u128);
impl_for_self!(usize);

impl_for_primitive!(i8 => Py<PyLong>);
impl_for_primitive!(i16 => Py<PyLong>);
impl_for_primitive!(i32 => Py<PyLong>);
impl_for_primitive!(i64 => Py<PyLong>);
impl_for_primitive!(i128 => Py<PyLong>);
impl_for_primitive!(isize => Py<PyLong>);
impl_for_primitive!(u8 => Py<PyLong>);
impl_for_primitive!(u16 => Py<PyLong>);
impl_for_primitive!(u32 => Py<PyLong>);
impl_for_primitive!(u64 => Py<PyLong>);
impl_for_primitive!(u128 => Py<PyLong>);
impl_for_primitive!(usize => Py<PyLong>);

// ==== Optional[T] ====

impl<'a, T, P> ToPython<Option<P>> for &'a Option<T>
where
    T: ToPython<P>,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<Option<P>> {
        self.as_ref().map(|inner| inner.to_python(py)).transpose()
    }
}

impl<T, P> ToPython<Option<P>> for Option<T>
where
    T: ToPython<P>,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<Option<P>> {
        <&Self as ToPython<Option<P>>>::to_python(&self, py)
    }
}

// ==== List ====

impl_for_self!(Py<PyList>);

impl<'a, T, P> ToPython<Vec<P>> for &'a [T]
where
    T: ToPython<P> + Clone,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<Vec<P>> {
        self.iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()
    }
}

impl<'a, T> ToPython<Py<PyList>> for &'a [T]
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        let elements = self
            .iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyList::new(py, elements).into_py(py))
    }
}

impl<'a, T> ToPython<Py<PyAny>> for &'a [T]
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyList>>>::to_python(self, py).map(|item| item.into_py(py))
    }
}

impl<T, P> ToPython<Vec<P>> for [T]
where
    T: ToPython<P> + Clone,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<Vec<P>> {
        <&Self as ToPython<Vec<P>>>::to_python(&self, py)
    }
}

impl<T> ToPython<Py<PyList>> for [T]
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        <&Self as ToPython<Py<PyList>>>::to_python(&self, py)
    }
}

impl<T> ToPython<Py<PyAny>> for [T]
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <&Self as ToPython<Py<PyAny>>>::to_python(&self, py)
    }
}

impl<T, P> ToPython<Vec<P>> for Vec<T>
where
    T: ToPython<P> + Clone,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<Vec<P>> {
        self.as_slice().to_python(py)
    }
}

impl<'a, T, P> ToPython<Vec<P>> for &'a Vec<T>
where
    T: ToPython<P> + Clone,
    P: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<Vec<P>> {
        self.as_slice().to_python(py)
    }
}

impl<T> ToPython<Py<PyList>> for Vec<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        self.as_slice().to_python(py)
    }
}

impl<'a, T> ToPython<Py<PyList>> for &'a Vec<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyList>> {
        self.as_slice().to_python(py)
    }
}

impl<T> ToPython<Py<PyAny>> for Vec<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.as_slice().to_python(py)
    }
}

impl<'a, T> ToPython<Py<PyAny>> for &'a Vec<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.as_slice().to_python(py)
    }
}

// ==== Set ====

impl_for_self!(Py<PySet>);

impl<'a, T, P, Hasher> ToPython<HashSet<P, Hasher>> for &'a HashSet<T, Hasher>
where
    T: ToPython<P> + Clone,
    P: ToPyObject + std::hash::Hash + Eq,
    Hasher: Default + BuildHasher,
{
    fn to_python(&self, py: Python) -> PyResult<HashSet<P, Hasher>> {
        self.iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<_>>()
    }
}

impl<T, P, Hasher> ToPython<HashSet<P, Hasher>> for HashSet<T, Hasher>
where
    T: ToPython<P> + Clone,
    P: ToPyObject + std::hash::Hash + Eq,
    Hasher: Default + BuildHasher,
{
    fn to_python(&self, py: Python) -> PyResult<HashSet<P, Hasher>> {
        <&Self as ToPython<HashSet<P, Hasher>>>::to_python(&self, py)
    }
}

impl<'a, T, Hasher> ToPython<Py<PySet>> for &'a HashSet<T, Hasher>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        // Using PySet::new seems to do extra cloning, so build manually.
        let set = PySet::empty(py)?;
        for item in *self {
            set.add(item.to_python(py)?)?;
        }
        Ok(set.into_py(py))
    }
}

impl<T, Hasher> ToPython<Py<PySet>> for HashSet<T, Hasher>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        <&Self as ToPython<Py<PySet>>>::to_python(&self, py)
    }
}

impl<'a, T, Hasher> ToPython<Py<PyAny>> for &'a HashSet<T, Hasher>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PySet>>>::to_python(self, py).map(|item| item.into_py(py))
    }
}

impl<T, Hasher> ToPython<Py<PyAny>> for HashSet<T, Hasher>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <&Self as ToPython<Py<PyAny>>>::to_python(&self, py)
    }
}

impl<'a, T, P> ToPython<BTreeSet<P>> for &'a BTreeSet<T>
where
    T: ToPython<P> + Clone,
    // Hash is required for the ToPyObject impl
    P: ToPyObject + Ord + std::hash::Hash,
{
    fn to_python(&self, py: Python) -> PyResult<BTreeSet<P>> {
        self.iter()
            .map(|item| item.to_python(py))
            .collect::<PyResult<_>>()
    }
}

impl<T, P> ToPython<BTreeSet<P>> for BTreeSet<T>
where
    T: ToPython<P> + Clone,
    P: ToPyObject + Ord + std::hash::Hash,
{
    fn to_python(&self, py: Python) -> PyResult<BTreeSet<P>> {
        <&Self as ToPython<BTreeSet<P>>>::to_python(&self, py)
    }
}

impl<'a, T> ToPython<Py<PySet>> for &'a BTreeSet<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        // Using PySet::new seems to do extra cloning, so build manually.
        let set = PySet::empty(py)?;
        for item in *self {
            set.add(item.to_python(py)?)?;
        }
        Ok(set.into_py(py))
    }
}

impl<T> ToPython<Py<PySet>> for BTreeSet<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PySet>> {
        <&Self as ToPython<Py<PySet>>>::to_python(&self, py)
    }
}

impl<'a, T> ToPython<Py<PyAny>> for &'a BTreeSet<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PySet>>>::to_python(self, py).map(|item| item.into_py(py))
    }
}

impl<T> ToPython<Py<PyAny>> for BTreeSet<T>
where
    T: ToPython<Py<PyAny>> + Clone,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <&Self as ToPython<Py<PyAny>>>::to_python(&self, py)
    }
}

// ==== String ====

impl_for_self!(Py<PyString>);
impl_for_self!(String);

private_impl_to_python_with_reference!(&self, py, str => Py<PyString> {
    Ok(PyString::new(py, self).into_py(py))
});

private_impl_to_python_pyany!(str => Py<PyString>);

private_impl_to_python_with_reference!(&self, py, String => Py<PyString> {
    self.as_str().to_python(py)
});

// ==== Time ====

impl_for_self!(Py<PyTime>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, (Time, Option<UtcOffset>) => Py<PyTime> {
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

#[cfg(feature = "time")]
private_impl_to_python_pyany!((Time, Option<UtcOffset>) => Py<PyTime>);

// ==== TzInfo ====

impl_for_self!(Py<PyTzInfo>);

#[cfg(feature = "time")]
private_impl_to_python_with_reference!(&self, py, UtcOffset => Py<PyTzInfo> {
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

#[cfg(feature = "time")]
private_impl_to_python_pyany!(UtcOffset => Py<PyTzInfo>);

// ==== IndexMap =====

#[cfg(feature = "indexmap")]
impl<'a, K1, K2, V1, V2, S> ToPython<IndexMap<K2, V2>> for &'a IndexMap<K1, V1, S>
where
    K1: ToPython<K2>,
    V1: ToPython<V2>,
    K2: ToPyObject + Eq + std::hash::Hash,
    V2: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<IndexMap<K2, V2>> {
        self.iter()
            .map(|(key, val)| {
                let key = key.to_python(py)?;
                let val = val.to_python(py)?;
                Ok((key, val))
            })
            .collect::<Result<_, _>>()
    }
}

impl<'a, K, V, S> ToPython<Py<PyDict>> for &'a IndexMap<K, V, S>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
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

impl<'a, K, V, S> ToPython<Py<PyAny>> for &'a IndexMap<K, V, S>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyDict>>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

impl<K1, K2, V1, V2, S> ToPython<IndexMap<K2, V2>> for IndexMap<K1, V1, S>
where
    K1: ToPython<K2>,
    V1: ToPython<V2>,
    K2: ToPyObject + Eq + std::hash::Hash,
    V2: ToPyObject,
{
    fn to_python(&self, py: Python) -> PyResult<IndexMap<K2, V2>> {
        <&Self as ToPython<IndexMap<K2, V2>>>::to_python(&self, py)
    }
}

impl<K, V, S> ToPython<Py<PyDict>> for IndexMap<K, V, S>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyDict>> {
        <&Self as ToPython<Py<PyDict>>>::to_python(&self, py)
    }
}

impl<K, V, S> ToPython<Py<PyAny>> for IndexMap<K, V, S>
where
    K: ToPython<Py<PyAny>> + std::fmt::Debug,
    V: ToPython<Py<PyAny>>,
{
    fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        <Self as ToPython<Py<PyDict>>>::to_python(self, py).map(|dict| dict.into_py(py))
    }
}

// ============ End Implementations ==============

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
// limitations under the License./

use std::ffi::c_double;

use num_complex::Complex;
use num_traits::{Float, FloatConst};
use pyo3::IntoPy;
use pyo3::{types::PyComplex, Py, PyAny, PyResult, Python};

use crate::{impl_for_self, ToPython};

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

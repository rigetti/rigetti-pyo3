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

use std::fmt::Display;
use std::os::raw::c_double;

use num_complex::Complex;
use num_traits::{Float, FloatConst};
use pyo3::{exceptions::PyFloatingPointError, types::PyComplex, IntoPy, Py, PyResult, Python};

use crate::py_try_from::{impl_try_from_self_python, PyAny, PyTryFrom};

impl_try_from_self_python!(PyComplex);

impl<F> PyTryFrom<Self> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(_py: Python, item: &Self) -> PyResult<Self> {
        Ok(*item)
    }
}

impl<F> PyTryFrom<Py<PyComplex>> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(py: Python, item: &Py<PyComplex>) -> PyResult<Self> {
        Self::py_try_from(py, item.as_ref(py))
    }
}

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

impl<F> PyTryFrom<PyAny> for Complex<F>
where
    F: Copy + Float + FloatConst + Into<c_double> + Display,
{
    fn py_try_from(py: Python, item: &PyAny) -> PyResult<Self> {
        let dict: &PyComplex = item.downcast()?;
        Self::py_try_from(py, dict)
    }
}

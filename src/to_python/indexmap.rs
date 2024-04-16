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

use indexmap::IndexMap;
use pyo3::{types::PyDict, IntoPy, Py, PyAny, PyResult, Python, ToPyObject};

use crate::ToPython;

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

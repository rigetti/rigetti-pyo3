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
use pyo3::{types::PyDict, Py, PyAny, PyResult, Python};

use crate::PyTryFrom;

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

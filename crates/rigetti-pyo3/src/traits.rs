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

//! Macros for implementing "dunder" methods based on traits.

/// Implement `__repr__` for a type that implements [`Debug`](std::fmt::Debug).
#[macro_export]
macro_rules! impl_repr {
    ($($name:ident),* $(,)?) => {
        $(
            $crate::maybe_add_cfg_stubs_gen_stub_pymethods! {
                #[$crate::pyo3::pymethods]
                impl $name {
                    /// Implements `__repr__` for Python in terms of the Rust
                    /// [`Debug`](std::fmt::Debug) implementation.
                    pub fn __repr__(&self) -> String {
                        format!("{self:?}")
                    }
                }
            }
        )*
    };
}

#[cfg(not(feature = "stubs"))]
#[doc(hidden)]
#[macro_export]
macro_rules! maybe_add_cfg_stubs_gen_stub_pymethods {
    ($($body:tt)*) => {
        $($body)*
    }
}

#[cfg(feature = "stubs")]
#[doc(hidden)]
#[macro_export]
macro_rules! maybe_add_cfg_stubs_gen_stub_pymethods {
    ($($body:tt)*) => {
        #[cfg_attr(feature = "stubs", $crate::pyo3_stub_gen::derive::gen_stub_pymethods)]
        $($body)*
    }
}

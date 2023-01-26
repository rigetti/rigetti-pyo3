use pyo3::{self, pymodule, types::PyModule, PyResult, Python};

pub mod rust {

    #[derive(Debug, thiserror::Error)]
    #[error("can't shave a yak with clogged clippers!")]
    pub struct CloggedClippersError;

    #[derive(Clone, Default)]
    pub struct Yak {
        is_shaved: bool,
    }

    impl Yak {
        pub fn new() -> Self {
            Self { is_shaved: false }
        }

        pub fn new_shaved() -> Self {
            Self { is_shaved: true }
        }

        pub fn is_shaved(&self) -> bool {
            self.is_shaved
        }

        pub fn shave_with(&mut self, tool: &mut CuttingTool) -> Result<(), CloggedClippersError> {
            if self.is_shaved() {
                return Ok(());
            }

            match tool {
                CuttingTool::Shears(_) => {
                    self.is_shaved = true;
                    Ok(())
                }
                CuttingTool::Clippers(clippers) => {
                    if clippers.is_clogged {
                        Err(CloggedClippersError)
                    } else {
                        clippers.is_clogged = true;
                        self.is_shaved = true;
                        Ok(())
                    }
                }
            }
        }
    }

    impl From<bool> for Yak {
        fn from(is_shaved: bool) -> Self {
            Self { is_shaved }
        }
    }

    impl From<Yak> for bool {
        fn from(yak: Yak) -> Self {
            yak.is_shaved
        }
    }

    #[derive(Clone, Default)]
    pub struct Clippers {
        is_clogged: bool,
    }

    impl Clippers {
        pub fn new() -> Self {
            Self { is_clogged: false }
        }

        pub fn unclog(&mut self) {
            self.is_clogged = false;
        }

        pub fn is_clogged(&self) -> bool {
            self.is_clogged
        }
    }

    #[derive(Clone, Default)]
    pub struct Shears;

    #[derive(Clone)]
    pub enum CuttingTool {
        Shears(Shears),
        Clippers(Clippers),
    }

    impl CuttingTool {
        pub fn is_clogged(&self) -> bool {
            matches!(self, Self::Clippers(Clippers { is_clogged: true }))
        }

        pub fn unclog(&mut self) {
            if let Self::Clippers(clippers) = self {
                clippers.unclog();
            }
        }
    }
}

pub mod python {
    use crate::rust::{
        Clippers, CloggedClippersError as RustCloggedClippers, CuttingTool, Shears, Yak,
    };
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::types::{PyBool, PyDict};
    use pyo3::{pymethods, IntoPy, Py, PyErr, PyResult, Python};
    use rigetti_pyo3::{
        create_init_submodule, impl_as_mut_for_wrapper, py_wrap_error, py_wrap_struct,
        py_wrap_type, py_wrap_union_enum, PyTryFrom, PyWrapper, PyWrapperMut, ToPython,
        ToPythonError,
    };

    create_init_submodule! {
        classes: [ PyYak, PyShears, PyClippers, PyCuttingTool ],
        errors: [ CloggedClippersError ],
    }

    py_wrap_error!(
        yak_shaving,
        // Renamed Rust error so the name can be used for the Python exception.
        RustCloggedClippers,
        // Name of Python exception.
        CloggedClippersError,
        PyRuntimeError
    );

    // Don't need to manually convert between `Yak` and `PyYak` -- use conversion blocks for
    // building from other types.
    py_wrap_struct! {
        PyYak(Yak) as "Yak" {
            py -> rs {
                py_dict: Py<PyDict> => Yak {
                    let is_shaved: &PyBool = py_dict.as_ref(py).as_mapping().get_item("is_shaved")?.downcast()?;
                    if is_shaved.is_true() {
                        Ok::<_, PyErr>(Yak::new_shaved())
                    } else {
                        Ok(Yak::new())
                    }
                },
                py_bool: Py<PyBool> => bool {
                    bool::py_try_from(py, &py_bool)
                }
            },
            rs -> py {
                yak: Yak => Py<PyDict> {
                    let dict = PyDict::new(py);
                    dict.set_item("is_shaved", yak.is_shaved())?;
                    Ok(dict.into_py(py))
                },
                b: bool => Py<PyBool> {
                    b.to_python(py)
                }
            }
        }
    }

    #[pymethods]
    impl PyYak {
        pub fn shave_with(&mut self, _py: Python<'_>, tool: &mut PyCuttingTool) -> PyResult<()> {
            self.as_inner_mut()
                .shave_with(tool.as_inner_mut())
                .map_err(RustCloggedClippers::to_py_err)
        }
    }

    py_wrap_type! {
        #[derive(Default)]
        PyShears(Shears) as "Shears";
    }

    impl_as_mut_for_wrapper!(PyShears);

    #[pymethods]
    impl PyShears {
        #[new]
        pub fn new() -> Self {
            Self::default()
        }
    }

    py_wrap_type! {
        #[derive(Default)]
        PyClippers(Clippers) as "Clippers";
    }

    impl_as_mut_for_wrapper!(PyClippers);

    #[pymethods]
    impl PyClippers {
        #[new]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn is_clogged(&self) -> bool {
            self.as_inner().is_clogged()
        }

        pub fn unclog(&mut self) {
            self.as_inner_mut().unclog()
        }
    }

    py_wrap_union_enum! {
        PyCuttingTool(CuttingTool) as "CuttingTool" {
            shears: Shears => PyShears,
            clippers: Clippers => PyClippers
        }
    }

    #[pymethods]
    impl PyCuttingTool {
        pub fn is_clogged(&self) -> bool {
            self.as_inner().is_clogged()
        }

        pub fn unclog(&mut self) {
            self.as_inner_mut().unclog()
        }
    }
}

#[pymodule]
fn yak_shaving(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    python::init_submodule("yak_shaving", py, m)
}

fn main() -> PyResult<()> {
    pyo3::append_to_inittab!(yak_shaving);
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let code = r#"
from yak_shaving import Yak, CuttingTool, Clippers, Shears, CloggedClippersError

shears = CuttingTool(Shears())
clippers = CuttingTool(Clippers())

yak1 = Yak(False)
yak2 = Yak(False)
yak3 = Yak(True)  # Already shaved
yak4 = Yak(False)
yak5 = Yak({ "is_shaved": False })

yak1.shave_with(shears)
yak2.shave_with(shears)

yak3.shave_with(clippers)
yak4.shave_with(clippers)
yak3.shave_with(clippers)

try:
    assert clippers.is_clogged()
    yak5.shave_with(clippers)
except CloggedClippersError:
    pass

clippers.unclog()
yak5.shave_with(clippers)
"#;

        PyModule::from_code(py, code, "example.py", "example")?;

        Ok(())
    })
}

#[test]
fn test_yak_shaving_example() {
    main().unwrap()
}

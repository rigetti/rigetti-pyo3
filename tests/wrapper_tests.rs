use pyo3::{self, pymodule, types::PyModule, PyResult, Python};

pub mod rust {
    #[derive(Clone, Copy)]
    pub enum TestEnum {
        One,
        Two,
    }

    #[derive(Clone, Copy)]
    pub struct TestStruct {
        pub test_enum: TestEnum,
    }
}

pub mod python {
    use super::rust::*;

    use pyo3::pymethods;
    use rigetti_pyo3::{create_init_submodule, py_wrap_data_struct, py_wrap_simple_enum};

    create_init_submodule! {
        classes: [ PyTestEnum, PyTestStruct ],
    }

    py_wrap_simple_enum! {
        PyTestEnum(TestEnum) as "TestEnum" {
            One,
            Two
        }
    }

    py_wrap_data_struct! {
        PyTestStruct(TestStruct) as "TestStruct" {
            test_enum: TestEnum => PyTestEnum
        }
    }

    #[pymethods]
    impl PyTestStruct {
        #[new]
        fn __new__() -> Self {
            Self(TestStruct {
                test_enum: TestEnum::One,
            })
        }
    }
}

#[pymodule]
fn wrapper_tests(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    python::init_submodule("wrapper_tests", py, m)
}

#[test]
fn test_enum_as_data_struct_member() {
    pyo3::append_to_inittab!(wrapper_tests);
    pyo3::prepare_freethreaded_python();
    let result: PyResult<()> = Python::with_gil(|py| {
        let code = r#"
from wrapper_tests import TestEnum, TestStruct

struct = TestStruct()

assert struct.test_enum == TestEnum.One

struct.test_enum = TestEnum.Two

assert struct.test_enum == TestEnum.Two
"#;
        PyModule::from_code(py, code, "example.py", "example")?;

        Ok(())
    });

    result.expect("python code should execute without issue")
}

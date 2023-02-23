use pyo3::{self, pymodule, types::PyModule, PyResult, Python};

pub mod rust {
    #[derive(Clone, Copy)]
    pub enum TestEnum {
        One,
        Two,
    }

    #[derive(Clone)]
    pub enum TestUnionEnum {
        Unit,
        String(String),
    }

    #[derive(Clone, Copy)]
    pub struct TestStruct {
        pub test_enum_unaliased: TestEnum,
        pub test_enum_aliased: TestEnum,
    }
}

pub mod python {
    use super::rust::*;

    use pyo3::pymethods;
    use rigetti_pyo3::{create_init_submodule, py_wrap_data_struct, py_wrap_simple_enum, py_wrap_union_enum};

    create_init_submodule! {
        classes: [ PyTestEnumUnaliased, PyTestEnumAliased, PyTestStruct, PyTestUnionEnum ],
    }

    py_wrap_union_enum! {
        PyTestUnionEnum(TestUnionEnum) as "TestUnionEnum" {
            unit: Unit,
            string: String => String
        }
    }

    py_wrap_simple_enum! {
        PyTestEnumUnaliased(TestEnum) as "TestEnumUnaliased" {
            One,
            Two
        }
    }

    py_wrap_simple_enum! {
        PyTestEnumAliased(TestEnum) as "TestEnumAliased" {
            One as NONE,
            Two as Two
        }
    }

    py_wrap_data_struct! {
        PyTestStruct(TestStruct) as "TestStruct" {
            test_enum_unaliased: TestEnum => PyTestEnumUnaliased,
            test_enum_aliased: TestEnum => PyTestEnumAliased
        }
    }

    #[pymethods]
    impl PyTestStruct {
        #[new]
        fn __new__() -> Self {
            Self(TestStruct {
                test_enum_unaliased: TestEnum::One,
                test_enum_aliased: TestEnum::One,
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
from wrapper_tests import TestEnumUnaliased, TestEnumAliased, TestStruct, TestUnionEnum

struct = TestStruct()

assert struct.test_enum_unaliased == TestEnumUnaliased.One
assert struct.test_enum_aliased == TestEnumAliased.NONE

struct.test_enum_unaliased = TestEnumUnaliased.Two
struct.test_enum_aliased = TestEnumAliased.Two

assert struct.test_enum_unaliased == TestEnumUnaliased.Two
assert struct.test_enum_aliased == TestEnumAliased.Two

assert TestUnionEnum.new_unit().is_unit()
"#;
        PyModule::from_code(py, code, "example.py", "example")?;

        Ok(())
    });

    result.expect("python code should execute without issue")
}

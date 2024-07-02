use pyo3::{types::PyModule, PyResult, Python};

// The code being tested is in a separate module so it can be expanded (with
// `cargo expand`) without expanding the contents of the tests themselves, which
// is useful when testing or debugging the macros defined in this crate.  This
// file must be in a subdirectory (`wrapper_tests/mod.rs` instead of
// `wrapper_tests.rs`) because the generated `.expanded.rs` file cannot be in
// the root `tests/` directory or `cargo test` will attempt to build it as a
// test case as well.
mod wrapper_tests;

#[test]
fn test_enum_as_data_struct_member() {
    wrapper_tests::append_to_inittab();
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

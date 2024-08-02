use pyo3::{types::PyModule, PyResult, Python};

// The code being tested is in a separate module so it can be expanded (see
// [`test_macro_expansion`]) without expanding the contents of the tests
// themselves.  You can also take advantage of this when manually using `cargo
// expand`, which may be useful when testing or debugging the macros defined in
// this crate.  This file must be in a subdirectory (`wrapper_tests/mod.rs`
// instead of `wrapper_tests.rs`) because the generated `.expanded.rs` file
// cannot be in the root `tests/` directory or `cargo test` will attempt to
// build it as a test case as well.
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

#[test]
fn test_macro_expansion() {
    // To regenerate the snapshot, run this test with the environment variable
    // `MACROTEST=overwrite`, or alternatively delete the generated
    // `tests/wrapper_tests/mod.expanded.rs` file and rerun this test.
    macrotest::expand_args(
        "tests/wrapper_tests/mod.rs",
        // We have to specify a specific OS target because until PyO3 v0.22,
        // PyO3 transitively depends on the
        // [rust-ctor](https://crates.io/crates/ctor) crate, which generates
        // different output on different OSes.  Once we're doing *that*, we have
        // to specify a specific Python ABI so that PyO3 doesn't get alarmed
        // about cross-compilation.  We pick the oldest availble option so that
        // we can be flexible with which Python interpreter is available on the
        // system.  This is all a minor headache.
        //
        // In particular, this means that if you are running these tests on a
        // different OS, you will need to install the specified target.  The
        // target is specifically chosen to be the one we use on CI, so CI does
        // not need an extra `rustup target add`, but some developers will.
        &[
            "--target",
            "x86_64-unknown-linux-gnu",
            "--no-default-features",
            "--features",
            "pyo3/abi3-py37",
        ],
    )
}

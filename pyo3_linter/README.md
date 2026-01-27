# PyO3 Linter

This Python package can be used to Rust code that uses PyO3 and `rigetti-pyo3`.

It's used within Rigetti repositories to find potential errors in the PyO3 bindings,
and to print information about the structure the bindings are expected to generate.
To see examples of that use, check the `quil-rs` repository's [lint script][quil-lint-script].

[quil-lint-script]: https://github.com/rigetti/quil-rs/blob/main/quil-rs/scripts/lint-quil-rs.py

This crate demonstrates example usage of the `pypropagate` macro. It defines example functions and
methods, which may wrap Rust async functions, and be called from Python. The generated Python
bindings are used in the Poetry package within this crate to assert that contexts are properly
set and propagated across the Python to Rust boundary.

## Testing

The following will build and test the example.

```shell
cargo make python-check-all
```

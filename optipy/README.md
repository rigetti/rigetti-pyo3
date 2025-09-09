# Optional PyO3 Features

This crate provides a procedural macro that removes [`pyo3`][] related macros.

It can be applied to an item using `#[cfg_attr(not(feature = "python"), optipy)]`.
To keep the `PyO3` attributes while stripping only the `pyo3_stub_gen` attributes,
you can write the attribute as `#[strip_pyo3(only_stubs)]`.
By default, the macro strips both.

## Why is this needed?

To use PyO3 effectively,
you're expected to wrap items with the procedural macros `#[pyclass]` and `#[pymodule]`.
You can feature-gate their application via e.g. `#[cfg_attr(feature = "python", pyclass)]`,
and that will work just fine in many cases.
It goes wrong, however, if you need to use any of `PyO3`'s attributes within that scope,
such as applying `#[getter]` to a method or `#[pyo3(name = "SomethingElse")]` to an `enum` variant;
`pyo3` can only process (and remove) those attributes if the feature is enabled, 
but if you wrap them in `cfg_attr`, then `pyo3` won't know how to process them.
See the following issues and PRs for more information:

- https://github.com/PyO3/pyo3/issues/780
- https://github.com/PyO3/pyo3/issues/1003
- https://github.com/PyO3/pyo3/pull/2786 

This crate takes [the suggestion][pr-suggestion] from the last PR listed above,
and strips `pyo3` attributes, so you can apply it when your feature is _not_ enabled. 

[`pyo3`]: https://github.com/PyO3/
[pr-suggestion]: https://github.com/PyO3/pyo3/pull/2786#issuecomment-1331207264

### What about stubs?

If you're also use `pyo3_stub_gen` to generate Python stub files,
this macro strips those attributes, too,
and by default, the macro strips both attributes of both creates.
To keep the `PyO3` attributes while stripping only the `pyo3_stub_gen` attributes,
you can write the attribute as `#[strip_pyo3(only_stubs)]`,
which is useful if you're using an additional feature specifically for stubs,
since that isn't needed for the final Python package.

## Usage

Generally, you'll want to apply this to code using a feature gate,
and if you're using this to strip `pyo3_stub_gen`, too, you'll likely have two features,
such as a `python` feature to generate the Python-specific bindings
and a `stubs` feature specifically for stub generation (and implies the `python` feature).

Here's how you can use this macro to conditionally remove the `pyo3`-related attributes:

```rust,ignore
#[cfg(not(feature = "python"))]
use optipy::strip_pyo3;
#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[cfg_attr(not(feature = "python"), optipy::strip_pyo3)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
struct ExampleStruct {
    #[pyo3(get)]
    x: usize,
}
```

You may find yourself writing `pyo3_stub_gen` attributes to, say, override a return type.
In that case, when building the bindings (using just the `python` feature)
you can choose to strip only the `pyo3_stub_gen` attributes while leaving `PyO3`'s.

```rust,ignore
enum ExampleEnum {
    Str(String),
    Num(isize),
    List(Py<PyList>),
}

#[cfg(feature = "python")]
mod python {
    use optipy::strip_pyo3;
    #[cfg(feature = "stubs")]
    use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

    #[cfg_attr(feature = "stubs", gen_stub_pyclass)]
    #[pyo3::pyclass]
    struct SomeStruct {
        value: ExampleEnum,
    }

    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pymethods)]
    #[pyo3::pymethods]
    impl SomeStruct {
        #[new] // We don't want to remove this!
        fn new<'py>(value: pyo3::Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
            let py = value.py();
            if let Ok(num) = value.downcast::<PyInt>() {
                (Self { value: ExampleEnum::Num(num.extract()?) }).into_pyobject(py)
            } else if let Ok(list) = value.downcast::<PyList>() {
                (Self { value: ExampleEnum::List(value.unbind().extract(py)?) }).into_pyobject(py)
            } else if let Ok(string) = value.extract::<String>() {
                (Self { value: ExampleEnum::Str(string) }).into_pyobject(py)
            } else {
                Err(PyTypeError::new_err("Not a valid value!"))
            }
        }

        // But we need to remove this when not building stubs.
        #[gen_stub(override_return_type(type_repr = "tuple[str, int, list]"))]
        fn __getnewargs__(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
            match &self.value {
                ExampleEnum::Str(value) => (value.clone(),).into_pyobject(py),
                ExampleEnum::Num(value) => (value,).into_pyobject(py),
                ExampleEnum::List(value) => (value,).into_pyobject(py),
            }
        }
    }
}
```


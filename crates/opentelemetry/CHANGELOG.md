## 0.11.1-pr97.0 (2026-08-18)

### Features

- visibility tokens for init_submodule

## 0.11.1-pr.0 (2026-08-18)

### Features

- visibility tokens for init_submodule

## 0.11.1-rc.0 (2026-08-18)

### Features

- visibility tokens for init_submodule

## 0.11.0 (2026-08-10)

### Breaking Changes

- support fieldless enums and data structs
- Update pyo3 to 0.19
- Update the `time` dependency and set an upper bound, increase MSRV to 1.67.0
- Update `pyo3` to 0.19, `impl_hash!` is now compatible with 32-bit architectures.
- Update pyo3 to 0.20.0 (#33)
- fully move all dependencies on pyo3 time types under our time feature + expose new abi3 feature for documentation
- update core-deps (#93)

### Features

- create main branch
- copy from internal repo with GitHub actions
- copy from internal repo with GitHub actions
- impl ToPython for Option
- impl conversion for std::time::Duration and str
- impl PyTryFrom for some Box<T> types
- impl conversion to/from Self
- allow exporting constants in create_init_submodule
- impl conversion traits for Box<T>
- add isize conversion for good measure
- add blanket implementations of ToPython and PyTryFrom for tuples of types that implement those traits
- Loosen time requirement (#37)
- Support IndexMap (#43)
- Add macros to create synchronous Python functions from async Rust functions. (#48)
- add documentation to all generated methods
- add documentation to all generated methods
- add `impl ToPython` and `impl PyTryFrom` for `internment::ArcIntern`
- add `ToPython` and `PyTryFrom` instances for `internment::ArcIntern` (#58)
- bump MSRV to support Cargo.lock v4
- migrate `optipy` to `rigetti-pyo3`
- adopt qcs-dependencies-client crate (#77)
- consolidate otel crates (#92)

### Fixes

- avoid chained comparison operator compile error
- correct trait bounds for PyComplex conversion
- impl ToPython for &Option<T>
- correctly feature-gate imports
- doc tests in wrappers.rs
- impl ToPython for &Box<T>
- add usize to py_try_from and to_python
- include unitless variants in is_x, allow creating unitless variants
- Correct module naming and registration
- Make impl_hash! compatible with 32-bit word size
- The __name__ property on submodules created with `create_init_submodule!` is set to the fully qualified path. (#35)
- specify license (force patch release) (#40)
- move all dependencies on pyo3 time types under our time feature
- move all dependencies on pyo3 time types under our time feature (#45)
- *actually* move all dependencies on pyo3 time types under our time feature
- trigger new release
- `impl_compare!` should not use outer attribute
- strip_pyo3 must visit const items (#80)
- force new release to publish rigetti-pyo3 with PyO3 0.28 (#88)
- update to qcs-core-deps (client) 0.4 (#91)
- knope 'release' step auth (#95)

## 0.11.0-rc.0 (2026-08-10)

### Breaking Changes

- support fieldless enums and data structs
- Update pyo3 to 0.19
- Update the `time` dependency and set an upper bound, increase MSRV to 1.67.0
- Update `pyo3` to 0.19, `impl_hash!` is now compatible with 32-bit architectures.
- Update pyo3 to 0.20.0 (#33)
- fully move all dependencies on pyo3 time types under our time feature + expose new abi3 feature for documentation
- update core-deps (#93)

### Features

- create main branch
- copy from internal repo with GitHub actions
- copy from internal repo with GitHub actions
- impl ToPython for Option
- impl conversion for std::time::Duration and str
- impl PyTryFrom for some Box<T> types
- impl conversion to/from Self
- allow exporting constants in create_init_submodule
- impl conversion traits for Box<T>
- add isize conversion for good measure
- add blanket implementations of ToPython and PyTryFrom for tuples of types that implement those traits
- Loosen time requirement (#37)
- Support IndexMap (#43)
- Add macros to create synchronous Python functions from async Rust functions. (#48)
- add documentation to all generated methods
- add documentation to all generated methods
- add `impl ToPython` and `impl PyTryFrom` for `internment::ArcIntern`
- add `ToPython` and `PyTryFrom` instances for `internment::ArcIntern` (#58)
- bump MSRV to support Cargo.lock v4
- migrate `optipy` to `rigetti-pyo3`
- adopt qcs-dependencies-client crate (#77)
- consolidate otel crates (#92)

### Fixes

- avoid chained comparison operator compile error
- correct trait bounds for PyComplex conversion
- impl ToPython for &Option<T>
- correctly feature-gate imports
- doc tests in wrappers.rs
- impl ToPython for &Box<T>
- add usize to py_try_from and to_python
- include unitless variants in is_x, allow creating unitless variants
- Correct module naming and registration
- Make impl_hash! compatible with 32-bit word size
- The __name__ property on submodules created with `create_init_submodule!` is set to the fully qualified path. (#35)
- specify license (force patch release) (#40)
- move all dependencies on pyo3 time types under our time feature
- move all dependencies on pyo3 time types under our time feature (#45)
- *actually* move all dependencies on pyo3 time types under our time feature
- trigger new release
- `impl_compare!` should not use outer attribute
- strip_pyo3 must visit const items (#80)
- force new release to publish rigetti-pyo3 with PyO3 0.28 (#88)
- update to qcs-core-deps (client) 0.4 (#91)
- knope 'release' step auth

## 0.10.0 (2026-08-10)

### Breaking Changes

- support fieldless enums and data structs
- Update pyo3 to 0.19
- Update the `time` dependency and set an upper bound, increase MSRV to 1.67.0
- Update `pyo3` to 0.19, `impl_hash!` is now compatible with 32-bit architectures.
- Update pyo3 to 0.20.0 (#33)
- fully move all dependencies on pyo3 time types under our time feature + expose new abi3 feature for documentation
- update core-deps (#93)

### Features

- create main branch
- copy from internal repo with GitHub actions
- copy from internal repo with GitHub actions
- impl ToPython for Option
- impl conversion for std::time::Duration and str
- impl PyTryFrom for some Box<T> types
- impl conversion to/from Self
- allow exporting constants in create_init_submodule
- impl conversion traits for Box<T>
- add isize conversion for good measure
- add blanket implementations of ToPython and PyTryFrom for tuples of types that implement those traits
- Loosen time requirement (#37)
- Support IndexMap (#43)
- Add macros to create synchronous Python functions from async Rust functions. (#48)
- add documentation to all generated methods
- add documentation to all generated methods
- add `impl ToPython` and `impl PyTryFrom` for `internment::ArcIntern`
- add `ToPython` and `PyTryFrom` instances for `internment::ArcIntern` (#58)
- bump MSRV to support Cargo.lock v4
- migrate `optipy` to `rigetti-pyo3`
- adopt qcs-dependencies-client crate (#77)
- consolidate otel crates (#92)

### Fixes

- avoid chained comparison operator compile error
- correct trait bounds for PyComplex conversion
- impl ToPython for &Option<T>
- correctly feature-gate imports
- doc tests in wrappers.rs
- impl ToPython for &Box<T>
- add usize to py_try_from and to_python
- include unitless variants in is_x, allow creating unitless variants
- Correct module naming and registration
- Make impl_hash! compatible with 32-bit word size
- The __name__ property on submodules created with `create_init_submodule!` is set to the fully qualified path. (#35)
- specify license (force patch release) (#40)
- move all dependencies on pyo3 time types under our time feature
- move all dependencies on pyo3 time types under our time feature (#45)
- *actually* move all dependencies on pyo3 time types under our time feature
- trigger new release
- `impl_compare!` should not use outer attribute
- strip_pyo3 must visit const items (#80)
- force new release to publish rigetti-pyo3 with PyO3 0.28 (#88)
- update to qcs-core-deps (client) 0.4 (#91)

## 0.9.0 (2026-07-21)

### Breaking Changes

- upgrade to PyO3 0.29

## 0.8.0 (2026-07-21)

### Breaking Changes

- update to pyo3-0.28

## 0.7.0 (2026-05-26)

### Breaking Changes

- use QCS core-deps; update OTEL version (#35)

## 0.6.0 (2026-02-09)

### Breaking Changes

- knope, release, and publish (#33)

## 0.5.0 (2026-02-05)

### Breaking Changes

#### update pyo3 (#29)

## 0.4.0 (2025-03-20)

### Breaking Changes

#### Update opentelemetry to v0.27.1 (#26)

## 0.3.4 (2024-10-07)

### Fixes

#### cargo categories

## 0.3.3 (2024-10-07)

### Fixes

#### update futures-util (#23)

## 0.3.2 (2024-10-07)

### Fixes

#### update opentelemetry dependencies

#### drop batch stdout support

#### use explicit extension-module features

#### ensure new line between traces data

#### update tracing subscriber stubs

#### use tls tonic features (#18)

## 0.3.1 (2023-12-18)

### Fixes

#### update opentelemetry-macros

#### ci release flow (#11)

## 0.3.0 (2023-12-16)

### Breaking Changes

#### initial implementation (#1)

#### update pyo3 (#10)

### Features

#### update macros with default ctx extraction failure behavior

### Fixes

#### print by default on python ctx extraction failure (#7)

## 0.2.0 (2023-11-29)

### Breaking Changes

#### initial implementation (#1)

### Fixes

#### print by default on python ctx extraction failure (#7)

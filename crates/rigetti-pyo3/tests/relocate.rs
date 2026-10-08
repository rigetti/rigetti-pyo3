// Copyright 2026 Rigetti Computing
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

//! Tests for [`rigetti_pyo3::stubs::relocate`], using the stubs of a dependency's classes that a
//! consumer package embeds.

// Test fixtures: the workspace-wide documentation lints are not useful here, and the fixtures are
// only used for the stubs they register.
#![allow(
    dead_code,
    missing_docs,
    missing_debug_implementations,
    clippy::missing_const_for_fn,
    clippy::missing_docs_in_private_items,
    clippy::unused_self
)]

use std::collections::BTreeSet;

use pyo3_stub_gen::{
    StubGenConfig, StubInfo,
    derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods},
    generate::ModuleReExport,
    reexport_module_members,
};
use rigetti_pyo3::stubs::{RelocateError, relocate};

/// The dependency's module that classes are relocated from.
const SOURCE: &str = "dep._dep.configuration";
/// A new module in the consumer.
const CLIENT: &str = "pkg._pkg.client";
/// An existing module in the consumer.
const EXISTING: &str = "pkg._pkg.existing";

/// The dependency's classes.
mod dep {
    use super::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};
    use pyo3::prelude::*;

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep", subclass)]
    struct Base;

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep.configuration", extends = Base)]
    struct Derived;

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep.configuration", from_py_object)]
    #[derive(Clone)]
    struct Server;

    #[gen_stub_pymethods]
    #[pymethods]
    impl Server {
        // Its default renders as `...`, qualified by the module of its type.
        #[staticmethod]
        #[pyo3(signature = (fallback = Server))]
        fn with_fallback(fallback: Self) -> Self {
            fallback
        }
    }

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep.configuration")]
    struct Session;

    #[gen_stub_pymethods]
    #[pymethods]
    impl Session {
        fn server(&self) -> Server {
            Server
        }
    }

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep.configuration")]
    struct Callback;

    #[gen_stub_pymethods]
    #[pymethods]
    impl Callback {
        #[new]
        fn new(
            #[gen_stub(override_type(
                type_repr = "collections.abc.Callable[[Server], str]",
                imports = ("collections.abc")
            ))]
            _function: Bound<'_, PyAny>,
        ) -> Self {
            Self
        }
    }

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep.configuration")]
    struct Unrelated;

    #[gen_stub_pyclass]
    #[pyclass(module = "dep._dep.configuration")]
    struct Loader;

    #[gen_stub_pymethods]
    #[pymethods]
    impl Loader {
        fn load(
            &self,
            #[gen_stub(override_type(type_repr = "other.Profile", imports = ("dep._dep.other")))]
            _profile: Bound<'_, PyAny>,
        ) {
        }
    }

    #[gen_stub_pyfunction(module = "dep._dep.configuration")]
    #[pyfunction]
    fn get_session() -> Session {
        Session
    }
}

/// The consumer's own classes.
mod pkg {
    use super::{gen_stub_pyclass, gen_stub_pyfunction};
    use pyo3::prelude::*;

    #[gen_stub_pyclass]
    #[pyclass(module = "pkg._pkg.existing")]
    struct Existing;

    #[gen_stub_pyfunction(module = "pkg._pkg")]
    #[gen_stub(override_return_type(
        type_repr = "typing.Optional[configuration.Session]",
        imports = ("typing", "dep._dep.configuration")
    ))]
    #[pyfunction]
    fn session() -> Option<Py<PyAny>> {
        None
    }
}

reexport_module_members!("pkg" from "pkg._pkg"; *);
reexport_module_members!("pkg.existing" from "pkg._pkg.existing");
// Explicit, though it happens to list every class in the module.
reexport_module_members!("pkg" from "pkg._pkg.existing"; "Existing");
reexport_module_members!("pkg" from "dep._dep.configuration"; "Server");
reexport_module_members!("dep.configuration" from "dep._dep.configuration");

fn stubs() -> StubInfo {
    StubInfo::from_project_root(
        "pkg._pkg".into(),
        ".".into(),
        true,
        StubGenConfig::default(),
    )
    .expect("stub info should build")
}

fn re_export<'a>(stubs: &'a StubInfo, module: &str, source: &str) -> &'a ModuleReExport {
    stubs.modules[module]
        .module_re_exports
        .iter()
        .find(|r| r.source_module == source)
        .unwrap_or_else(|| panic!("{module} should re-export from {source}"))
}

fn has(re_export: &ModuleReExport, name: &str) -> bool {
    re_export.items.iter().any(|item| item == name)
}

#[test]
fn moves_closure_and_rewrites_references() {
    let mut stubs = stubs();
    let moved = relocate(&mut stubs, SOURCE, &["Session", "Callback"], CLIENT)
        .expect("relocation should succeed");
    assert_eq!(moved, BTreeSet::from(["Callback", "Server", "Session"]));

    let client = stubs.modules[CLIENT].to_string();
    assert!(client.contains("class Session"), "{client}");
    assert!(client.contains("def server(self) -> Server"), "{client}");
    assert!(client.contains("fallback: Server = ...)"), "{client}");
    // Found through the hand-written type.
    assert!(client.contains("class Server"), "{client}");
    assert!(client.contains("Callable[[Server], str]"), "{client}");
    assert!(!client.contains("configuration"), "{client}");

    let source = stubs.modules[SOURCE].to_string();
    assert!(source.contains("class Unrelated"), "{source}");
    assert!(!source.contains("class Session"), "{source}");
    assert!(source.contains("from pkg._pkg import client"), "{source}");
    assert!(
        source.contains("def get_session() -> client.Session"),
        "{source}"
    );

    let root = stubs.modules["pkg._pkg"].to_string();
    assert!(
        root.contains("def session() -> typing.Optional[client.Session]"),
        "{root}"
    );
    assert!(root.contains("from . import client"), "{root}");
    assert!(!root.contains("configuration"), "{root}");
}

#[test]
fn registers_new_target_module() {
    let mut stubs = stubs();
    relocate(&mut stubs, SOURCE, &["Server"], CLIENT).expect("relocation should succeed");

    assert!(stubs.modules["pkg._pkg"].submodules.contains("client"));
    assert!(has(re_export(&stubs, "pkg", "pkg._pkg"), "client"));
}

#[test]
fn updates_re_exports() {
    let mut stubs = stubs();
    relocate(&mut stubs, SOURCE, &["Server"], EXISTING).expect("relocation should succeed");

    // `*` re-exports of the target gain the class, but explicit ones don't.
    assert!(has(re_export(&stubs, "pkg.existing", EXISTING), "Server"));
    let explicit = re_export(&stubs, "pkg", EXISTING);
    assert!(has(explicit, "Existing"));

    // `*` re-exports of the source lose it, and explicit ones now come from the target.
    let wildcard = re_export(&stubs, "dep.configuration", SOURCE);
    assert!(!has(wildcard, "Server"), "{wildcard:?}");
    assert!(has(wildcard, "Unrelated"), "{wildcard:?}");
    assert!(!has(re_export(&stubs, "pkg", SOURCE), "Server"));
    assert!(has(explicit, "Server"), "{explicit:?}");
}

#[test]
fn explicit_re_exports_are_not_extended() {
    let mut stubs = stubs();
    relocate(&mut stubs, SOURCE, &["Unrelated"], EXISTING).expect("relocation should succeed");

    assert!(has(
        re_export(&stubs, "pkg.existing", EXISTING),
        "Unrelated"
    ));
    assert!(!has(re_export(&stubs, "pkg", EXISTING), "Unrelated"));
}

#[test]
fn rejects_same_module() {
    let mut stubs = stubs();
    let before = stubs.clone();
    assert!(matches!(
        relocate(&mut stubs, SOURCE, &["Server"], SOURCE),
        Err(RelocateError::SameModule(_))
    ));
    assert_eq!(stubs, before);
}

#[test]
fn rejects_missing_module() {
    assert!(matches!(
        relocate(&mut stubs(), "dep._dep.missing", &["Server"], CLIENT),
        Err(RelocateError::MissingModule(_))
    ));
}

#[test]
fn rejects_unknown_class() {
    assert!(matches!(
        relocate(&mut stubs(), SOURCE, &["NotAClass"], CLIENT),
        Err(RelocateError::UnknownClass { class, .. }) if class == "NotAClass"
    ));
}

#[test]
fn rejects_foreign_reference() {
    let mut stubs = stubs();
    let before = stubs.clone();
    assert!(matches!(
        relocate(&mut stubs, SOURCE, &["Derived"], CLIENT),
        Err(RelocateError::ForeignReference { class, module, .. })
            if class == "Derived" && module == "dep._dep"
    ));
    assert_eq!(stubs, before);
}

#[test]
fn rejects_foreign_import() {
    assert!(matches!(
        relocate(&mut stubs(), SOURCE, &["Loader"], CLIENT),
        Err(RelocateError::ForeignReference { class, module, .. })
            if class == "Loader" && module == "dep._dep.other"
    ));
}

#[test]
fn ignores_empty_request() {
    let mut stubs = stubs();
    let before = stubs.clone();
    let moved = relocate(&mut stubs, SOURCE, &[], CLIENT).expect("relocation should succeed");
    assert!(moved.is_empty());
    assert_eq!(stubs, before);
}

#[test]
fn rejects_name_conflict_in_wildcard_re_export() {
    let mut stubs = stubs();
    let function = stubs.modules[SOURCE].function["get_session"].clone();
    stubs
        .modules
        .get_mut("pkg.existing")
        .expect("re-exporting module should be present")
        .function
        .insert("Server", function);
    let before = stubs.clone();

    assert!(matches!(
        relocate(&mut stubs, SOURCE, &["Server"], EXISTING),
        Err(RelocateError::NameConflict { name, module })
            if name == "Server" && module == "pkg.existing"
    ));
    assert_eq!(stubs, before);
}

#[test]
fn rejects_name_conflict() {
    let mut stubs = stubs();
    let function = stubs.modules[SOURCE].function["get_session"].clone();
    stubs
        .modules
        .get_mut(EXISTING)
        .expect("existing module should be present")
        .function
        .insert("Server", function);
    let before = stubs.clone();

    assert!(matches!(
        relocate(&mut stubs, SOURCE, &["Server"], EXISTING),
        Err(RelocateError::NameConflict { name, .. }) if name == "Server"
    ));
    assert_eq!(stubs, before);
}

#[test]
fn rejects_target_outside_module_path() {
    let mut stubs = stubs();
    let before = stubs.clone();
    assert!(matches!(
        relocate(&mut stubs, SOURCE, &["Server"], "pkg.client"),
        Err(RelocateError::OutsideModulePath { module, root })
            if module == "pkg.client" && root == "pkg._pkg"
    ));
    assert_eq!(stubs, before);
}

#[test]
fn moves_all_entries() {
    let mut stubs = stubs();
    let source = stubs
        .modules
        .get_mut(SOURCE)
        .expect("source module should be present");
    source.verbatim_all_entries.insert("Server".to_string());
    source
        .excluded_all_entries
        .extend(["Session".to_string(), "Unrelated".to_string()]);

    relocate(&mut stubs, SOURCE, &["Session"], CLIENT).expect("relocation should succeed");

    let source = &stubs.modules[SOURCE];
    assert!(source.verbatim_all_entries.is_empty());
    assert_eq!(
        source.excluded_all_entries,
        BTreeSet::from(["Unrelated".to_string()])
    );
    let client = &stubs.modules[CLIENT];
    assert_eq!(
        client.verbatim_all_entries,
        BTreeSet::from(["Server".to_string()])
    );
    assert_eq!(
        client.excluded_all_entries,
        BTreeSet::from(["Session".to_string()])
    );
}

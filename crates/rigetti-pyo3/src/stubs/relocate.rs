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

//! Move class stubs from one module to another; see [`relocate`].

use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
};

use pyo3_stub_gen::{
    ImportRef, ModuleRef, StubInfo, TypeInfo,
    generate::{
        ClassDef, EnumDef, MemberDef, MethodDef, Module, ModuleReExport, Parameter,
        ParameterDefault, Parameters,
    },
    inventory,
    type_info::{ReexportItems, ReexportModuleMembers},
};

/// Errors that can occur while relocating stubs.
#[derive(Debug, thiserror::Error)]
pub enum RelocateError {
    /// The stubs have no source module, so nothing can be relocated.
    #[error("stubs have no `{0}` module")]
    MissingModule(String),
    /// The target module is the source module.
    #[error("cannot relocate classes from `{0}` into itself")]
    SameModule(String),
    /// A requested class isn't in the source module.
    #[error("class `{class}` is not defined in `{module}`")]
    UnknownClass {
        /// The requested class.
        class: String,
        /// The source module.
        module: String,
    },
    /// A relocated class refers to something in its package that can't be relocated with it,
    /// which the target's stubs wouldn't be able to resolve.
    #[error("relocated class `{class}` refers to `{name}` in `{module}`, which can't be relocated")]
    ForeignReference {
        /// The relocated class.
        class: String,
        /// The name it refers to.
        name: String,
        /// The module that defines `name`.
        module: String,
    },
    /// The target module already has something with the same name as a relocated class.
    #[error("`{module}` already defines or imports `{name}`")]
    NameConflict {
        /// The relocated class.
        name: String,
        /// The target module.
        module: String,
    },
}

/// Move the stubs for `classes`, and every class or enum in `source_module` they refer to, into
/// `target_module`, and rewrite all references to them anywhere in `stubs`.
///
/// This is for a crate that compiles another crate's `#[pyclass]`es into its own extension module
/// (rather than depending on the other crate's Python package), and registers them in one of its
/// own modules. `pyo3_stub_gen` gathers every linked crate's stubs, so without this they would
/// describe those classes as living in the other package.
///
/// Returns the names of every relocated class. The caller must register each of them in
/// `target_module` at runtime, or its stubs won't match its module. Run this before
/// [`sort`](super::sort).
///
/// Re-exports are updated to match: `*` re-exports of `target_module` gain the relocated classes,
/// `*` re-exports of `source_module` lose them, and explicit re-exports of them from
/// `source_module` re-export them from `target_module` instead. A new `target_module` is added to
/// its parent's submodules.
///
/// # Errors
///
/// See [`RelocateError`]. `stubs` is unchanged when this fails.
///
/// # Example
///
/// ```rust,no_run
/// # fn stub_info() -> pyo3_stub_gen::Result<pyo3_stub_gen::StubInfo> { unimplemented!() }
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut stubs = stub_info()?;
///     rigetti_pyo3::stubs::relocate(
///         &mut stubs,
///         "dependency._dependency.client",
///         &["Client"],
///         "my_package._my_package.client",
///     )?;
///     rigetti_pyo3::stubs::sort(&mut stubs);
///     stubs.generate()?;
///     Ok(())
/// }
/// ```
pub fn relocate(
    stubs: &mut StubInfo,
    source_module: &str,
    classes: &[&str],
    target_module: &'static str,
) -> Result<BTreeSet<&'static str>, RelocateError> {
    if target_module == source_module {
        return Err(RelocateError::SameModule(source_module.to_string()));
    }
    let taken = stubs
        .modules
        .get(target_module)
        .map(|target| names(target, source_module))
        .unwrap_or_default();
    let source = stubs
        .modules
        .get_mut(source_module)
        .ok_or_else(|| RelocateError::MissingModule(source_module.to_string()))?;
    let moved = closure(source, classes, target_module)?;
    if let Some(name) = moved.iter().find(|name| taken.contains(**name)) {
        return Err(RelocateError::NameConflict {
            name: (*name).to_string(),
            module: target_module.to_string(),
        });
    }

    let class_ids: Vec<TypeId> = source
        .class
        .iter()
        .filter(|(_, class)| moved.contains(class.name))
        .map(|(id, _)| *id)
        .collect();
    let enum_ids: Vec<TypeId> = source
        .enum_
        .iter()
        .filter(|(_, enum_)| moved.contains(enum_.name))
        .map(|(id, _)| *id)
        .collect();
    let classes: Vec<_> = class_ids
        .iter()
        .filter_map(|id| source.class.remove_entry(id))
        .collect();
    let enums: Vec<_> = enum_ids
        .iter()
        .filter_map(|id| source.enum_.remove_entry(id))
        .collect();

    let is_new = !stubs.modules.contains_key(target_module);
    let default_module_name = stubs.default_module_name.clone();
    let target = stubs
        .modules
        .entry(target_module.to_string())
        .or_insert_with(|| Module {
            name: target_module.to_string(),
            default_module_name,
            ..Module::default()
        });
    for (id, mut class) in classes {
        set_module(&mut class, target_module);
        target.class.insert(id, class);
    }
    for (id, mut enum_) in enums {
        enum_.module = Some(target_module);
        target.enum_.insert(id, enum_);
    }

    if is_new {
        register_submodule(stubs, target_module);
    }
    let public: Vec<&str> = moved
        .iter()
        .copied()
        .filter(|name| !name.starts_with('_'))
        .collect();
    extend_wildcards(stubs, target_module, &public);
    redirect_re_exports(stubs, source_module, &moved, target_module);

    for (name, module) in &mut stubs.modules {
        visit_module(
            module,
            &mut Rename {
                source: source_module,
                to: target_module,
                moved: &moved,
                in_module: name,
            },
        );
    }

    Ok(moved)
}

/// Find `classes` and every class or enum in `module` they transitively refer to.
fn closure(
    module: &mut Module,
    classes: &[&str],
    target_module: &str,
) -> Result<BTreeSet<&'static str>, RelocateError> {
    let defined: BTreeMap<&'static str, Item> = module
        .class
        .iter()
        .map(|(id, class)| (class.name, Item::Class(*id)))
        .chain(
            module
                .enum_
                .iter()
                .map(|(id, enum_)| (enum_.name, Item::Enum(*id))),
        )
        .collect();
    let source = module.name.clone();
    let package = source.split('.').next().unwrap_or_default();
    let mut moved = BTreeSet::new();
    let mut queue: Vec<String> = classes.iter().map(ToString::to_string).collect();

    while let Some(name) = queue.pop() {
        let Some((&name, item)) = defined.get_key_value(name.as_str()) else {
            return Err(RelocateError::UnknownClass {
                class: name,
                module: source,
            });
        };
        if !moved.insert(name) {
            continue;
        }

        let mut references = References {
            source: &source,
            package,
            defined: &defined,
            target_module,
            found: Vec::new(),
            foreign: None,
        };
        match item {
            Item::Class(id) => {
                if let Some(class) = module.class.get_mut(id) {
                    visit_class(class, &mut references);
                }
            }
            Item::Enum(id) => {
                if let Some(enum_) = module.enum_.get_mut(id) {
                    visit_enum(enum_, &mut references);
                }
            }
        }
        if let Some((name_ref, module)) = references.foreign {
            return Err(RelocateError::ForeignReference {
                class: name.to_string(),
                name: name_ref,
                module,
            });
        }
        queue.extend(references.found);
    }

    Ok(moved)
}

/// Something that can be relocated.
#[derive(Clone, Copy, Debug)]
enum Item {
    /// A class, by its key in [`Module::class`].
    Class(TypeId),
    /// An enum, by its key in [`Module::enum_`].
    Enum(TypeId),
}

/// Everything `module` defines or imports by name. Re-exports from `source_module` are left out,
/// since relocation redirects them.
fn names(module: &Module, source_module: &str) -> BTreeSet<String> {
    let re_exported = module
        .module_re_exports
        .iter()
        .filter(|re_export| re_export.source_module != source_module)
        .flat_map(|re_export| re_export.items.iter().map(String::as_str));
    module
        .class
        .values()
        .map(|class| class.name)
        .chain(module.enum_.values().map(|enum_| enum_.name))
        .chain(module.function.keys().copied())
        .chain(module.variables.keys().copied())
        .chain(module.type_aliases.keys().copied())
        .chain(module.submodules.iter().map(String::as_str))
        .chain(re_exported)
        .map(ToString::to_string)
        .collect()
}

/// Set the module of `class` and the classes nested in it.
fn set_module(class: &mut ClassDef, module: &'static str) {
    class.module = Some(module);
    for nested in &mut class.classes {
        set_module(nested, module);
    }
}

/// Add a new module to its ancestors' submodules, as the stubs builder does for every module it
/// finds before relocation.
fn register_submodule(stubs: &mut StubInfo, module: &str) {
    let mut child = module;
    while let Some((parent, name)) = child.rsplit_once('.') {
        if !is_pyo3_generated(stubs, parent) {
            break;
        }
        let default_module_name = stubs.default_module_name.clone();
        let parent_module = stubs
            .modules
            .entry(parent.to_string())
            .or_insert_with(|| Module {
                name: parent.to_string(),
                default_module_name,
                ..Module::default()
            });
        if !parent_module.submodules.insert(name.to_string()) {
            break;
        }
        if !name.starts_with('_') {
            extend_wildcards(stubs, parent, &[name]);
        }
        child = parent;
    }
}

/// Mirrors `StubInfo::is_pyo3_generated`, which is private.
fn is_pyo3_generated(stubs: &StubInfo, module: &str) -> bool {
    let module = module.replace('-', "_");
    let root = stubs.default_module_name.replace('-', "_");
    !stubs.is_mixed_layout || module == root || module.starts_with(&format!("{root}."))
}

/// Add `names` to every `*` re-export from `source`.
fn extend_wildcards(stubs: &mut StubInfo, source: &str, names: &[&str]) {
    for module in stubs.modules.values_mut() {
        let wildcards = wildcard_flags(module);
        for (re_export, is_wildcard) in module.module_re_exports.iter_mut().zip(wildcards) {
            if is_wildcard && re_export.source_module == source {
                add_items(re_export, names.iter().copied());
            }
        }
    }
}

/// Re-exports of moved classes from `source_module` would import names it no longer has.
/// Drop them from `*` re-exports, and point explicitly named ones at `target_module` instead.
fn redirect_re_exports(
    stubs: &mut StubInfo,
    source_module: &str,
    moved: &BTreeSet<&str>,
    target_module: &str,
) {
    for module in stubs.modules.values_mut() {
        let wildcards = wildcard_flags(module);
        let mut redirected = Vec::new();
        for (re_export, is_wildcard) in module.module_re_exports.iter_mut().zip(wildcards) {
            if re_export.source_module != source_module {
                continue;
            }
            // Emptied re-exports are kept (and render as nothing) so `wildcard_flags` still lines
            // up with the declarations.
            re_export.items.retain(|item| {
                let is_moved = moved.contains(item.as_str());
                if is_moved && !is_wildcard {
                    redirected.push(item.clone());
                }
                !is_moved
            });
        }
        if redirected.is_empty() || module.name == target_module {
            continue;
        }
        match module
            .module_re_exports
            .iter_mut()
            .find(|re_export| re_export.source_module == target_module)
        {
            Some(re_export) => add_items(re_export, redirected.iter().map(String::as_str)),
            None => module.module_re_exports.push(ModuleReExport {
                source_module: target_module.to_string(),
                items: redirected,
                additional_items: Vec::new(),
            }),
        }
    }
}

/// Add each of `names` that `re_export` doesn't already have.
fn add_items<'a>(re_export: &mut ModuleReExport, names: impl IntoIterator<Item = &'a str>) {
    for name in names {
        if !re_export.items.iter().any(|item| item == name) {
            re_export.items.push(name.to_string());
        }
    }
}

/// Whether each of `module`'s re-exports was declared with `*`.
///
/// `*` is resolved to a list when the stubs are built, so this pairs each re-export with its
/// declaration. The builder adds them in declaration order, and nothing else adds them before
/// [`relocate`] runs; the ones it adds come last and aren't wildcards.
fn wildcard_flags(module: &Module) -> Vec<bool> {
    let mut declared = inventory::iter::<ReexportModuleMembers>
        .into_iter()
        .filter(|declared| declared.target_module == module.name);
    module
        .module_re_exports
        .iter()
        .map(|re_export| {
            declared.next().is_some_and(|declared| {
                declared.source_module == re_export.source_module
                    && !matches!(declared.items, ReexportItems::Explicit(_))
            })
        })
        .collect()
}

/// The last component of a dotted module path, which is how stubs qualify names from it.
fn last_component(module: &str) -> &str {
    module.rsplit('.').next().unwrap_or(module)
}

/// Whether `module` is `package` or one of its submodules.
fn is_in_package(module: &str, package: &str) -> bool {
    module
        .strip_prefix(package)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

/// Whether `info` was built against `source`, so a qualified name using its last component means
/// that module rather than some other module with the same last component.
fn refers_to(info: &TypeInfo, source: &str) -> bool {
    let is_source = |module: &ModuleRef| module.get() == Some(source);
    info.source_module.as_ref().is_some_and(is_source)
        || info.type_refs.values().any(|r| is_source(&r.module))
        || info.import.iter().any(|import| match import {
            ImportRef::Module(module) => is_source(module),
            ImportRef::Type(type_ref) => is_source(&type_ref.module),
        })
}

/// Whether `ident` is imported by name from some module other than `source`.
fn imported_elsewhere(info: &TypeInfo, ident: &str, source: &str) -> bool {
    info.import.iter().any(|import| {
        matches!(import, ImportRef::Type(t) if t.name == ident && t.module.get() != Some(source))
    })
}

/// Whether `c` can be part of an identifier or dotted path.
fn is_path_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.'
}

/// Whether `path` is an identifier or dotted path, rather than e.g. a number.
fn is_path(path: &str) -> bool {
    path.starts_with(|c: char| c.is_alphabetic() || c == '_')
}

/// The identifiers and dotted paths in a type expression.
fn paths(expr: &str) -> impl Iterator<Item = &str> {
    expr.split(|c: char| !is_path_char(c))
        .filter(|path| is_path(path))
}

/// Replace each identifier or dotted path in `expr` for which `f` returns `Some`.
fn map_paths(expr: &str, mut f: impl FnMut(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(expr.len());
    let mut rest = expr;
    while !rest.is_empty() {
        let end = rest.find(|c: char| !is_path_char(c)).unwrap_or(rest.len());
        let (path, tail) = rest.split_at(end);
        match Some(path).filter(|path| is_path(path)).and_then(&mut f) {
            Some(replacement) => out.push_str(&replacement),
            None => out.push_str(path),
        }
        let sep = tail.find(is_path_char).unwrap_or(tail.len());
        out.push_str(&tail[..sep]);
        rest = &tail[sep..];
    }
    out
}

/// The name a path in `info` refers to if it's something in `source`, and whether the path is
/// qualified. A bare name only resolves to `source` when the expression is written in `source`.
fn source_ident<'a>(
    path: &'a str,
    info: &TypeInfo,
    source: &str,
    in_source: bool,
) -> Option<(&'a str, bool)> {
    match path.split_once('.') {
        None if in_source && !imported_elsewhere(info, path, source) => Some((path, false)),
        Some((head, rest)) if head == last_component(source) && refers_to(info, source) => {
            Some((rest.split('.').next().unwrap_or(rest), true))
        }
        _ => None,
    }
}

/// Something that inspects or rewrites the types in stubs.
trait Visitor {
    /// Visit a type annotation.
    fn type_info(&mut self, info: &mut TypeInfo);
    /// Visit a parameter's default value.
    fn default(&mut self, default: &mut ParameterDefault);
}

/// Collects what a class in the source module refers to.
#[derive(Debug)]
struct References<'a> {
    /// The source module.
    source: &'a str,
    /// The top-level package of the source module.
    package: &'a str,
    /// Everything in the source module that can be relocated.
    defined: &'a BTreeMap<&'static str, Item>,
    /// The target module, which classes may already have been relocated into.
    target_module: &'a str,
    /// Names of the source module's classes and enums that were referred to.
    found: Vec<String>,
    /// The first name referred to elsewhere in the package, and its module.
    foreign: Option<(String, String)>,
}

impl References<'_> {
    /// Record a reference to `name` in `module`, which isn't relocated.
    fn foreign(&mut self, name: &str, module: &str) {
        self.foreign
            .get_or_insert_with(|| (name.to_string(), module.to_string()));
    }

    /// Whether `module` is in the package, but not the source or target module.
    fn is_foreign(&self, module: &str) -> bool {
        module != self.source && module != self.target_module && is_in_package(module, self.package)
    }
}

impl Visitor for References<'_> {
    fn type_info(&mut self, info: &mut TypeInfo) {
        for (ident, type_ref) in &info.type_refs {
            match type_ref.module.get() {
                Some(module) if module == self.source => {
                    if self.defined.contains_key(ident.as_str()) {
                        self.found.push(ident.clone());
                    } else {
                        self.foreign(ident, module);
                    }
                }
                Some(module) if self.is_foreign(module) => self.foreign(ident, module),
                _ => {}
            }
        }
        if let Some(module) = info.source_module.as_ref().and_then(ModuleRef::get)
            && self.is_foreign(module)
        {
            self.foreign(&info.name, module);
        }

        // Types written out by hand (e.g. `override_type`) have no `type_refs`, so look for
        // names of this module in the expression itself. These classes are written in the source
        // module, so bare names resolve there.
        for path in paths(&info.name) {
            if let Some((ident, _)) = source_ident(path, info, self.source, true)
                && self.defined.contains_key(ident)
            {
                self.found.push(ident.to_string());
            }
        }
    }

    fn default(&mut self, default: &mut ParameterDefault) {
        if let ParameterDefault::Expr {
            value,
            source_module: Some(module),
        } = default
            && module.get() == Some(self.source)
        {
            let head = value.split(['.', '(']).next().unwrap_or_default();
            if self.defined.contains_key(head) {
                self.found.push(head.to_string());
            }
        }
    }
}

/// Rewrites references to relocated classes in the stubs for `in_module`.
#[derive(Debug)]
struct Rename<'a> {
    /// The module the classes were relocated from.
    source: &'a str,
    /// The module the classes were relocated to.
    to: &'static str,
    /// The relocated classes.
    moved: &'a BTreeSet<&'static str>,
    /// The module being rewritten.
    in_module: &'a str,
}

impl Rename<'_> {
    /// Whether `module` is the source module.
    fn is_source(&self, module: &ModuleRef) -> bool {
        module.get() == Some(self.source)
    }

    /// The target module, as a [`ModuleRef`].
    fn to(&self) -> ModuleRef {
        ModuleRef::Named(self.to.to_string())
    }
}

impl Visitor for Rename<'_> {
    fn type_info(&mut self, info: &mut TypeInfo) {
        let in_source = self.in_module == self.source;
        let in_target = self.in_module == self.to;
        let mut touched = false;

        for (ident, type_ref) in &mut info.type_refs {
            if self.is_source(&type_ref.module) && self.moved.contains(ident.as_str()) {
                type_ref.module = self.to();
                touched = true;
            }
        }

        let to_component = last_component(self.to);
        let name = map_paths(&info.name, |path| {
            let (ident, qualified) = source_ident(path, info, self.source, in_source)?;
            if !self.moved.contains(ident) {
                return None;
            }
            // `path` is either `Class...` or `source.Class...`.
            let rest = if qualified {
                path.split_once('.').map_or(path, |(_, rest)| rest)
            } else {
                path
            };
            Some(if in_target {
                rest.to_string()
            } else {
                format!("{to_component}.{rest}")
            })
        });
        let renamed = name != info.name;

        let import: Vec<ImportRef> = info.import.drain().collect();
        for import in import {
            let import = match import {
                ImportRef::Type(mut t)
                    if self.is_source(&t.module) && self.moved.contains(t.name.as_str()) =>
                {
                    t.module = self.to();
                    touched = true;
                    ImportRef::Type(t)
                }
                other => other,
            };
            info.import.insert(import);
        }

        if !touched && !renamed {
            return;
        }
        info.name = name;

        let source_component = last_component(self.source);
        let still_refers = info.type_refs.values().any(|r| self.is_source(&r.module))
            || info
                .import
                .iter()
                .any(|import| matches!(import, ImportRef::Type(t) if self.is_source(&t.module)))
            || paths(&info.name).any(|path| {
                path.split_once('.')
                    .is_some_and(|(head, _)| head == source_component)
            });
        let is_single_path = info.name.chars().all(is_path_char);
        if info
            .source_module
            .as_ref()
            .is_some_and(|m| self.is_source(m))
            && (!still_refers || (renamed && is_single_path))
        {
            info.source_module = Some(self.to());
        }
        if !still_refers
            && !info
                .source_module
                .as_ref()
                .is_some_and(|m| self.is_source(m))
        {
            info.import.remove(&ImportRef::Module(ModuleRef::Named(
                self.source.to_string(),
            )));
        }
        if !in_target {
            info.import.insert(ImportRef::Module(self.to()));
        }
    }

    fn default(&mut self, default: &mut ParameterDefault) {
        if let ParameterDefault::Expr {
            value,
            source_module: Some(module),
        } = default
        {
            let head = value.split(['.', '(']).next().unwrap_or_default();
            if self.is_source(module) && self.moved.contains(head) {
                *module = self.to();
            }
        }
    }
}

/// Visit every type in `module`.
fn visit_module(module: &mut Module, visitor: &mut impl Visitor) {
    for class in module.class.values_mut() {
        visit_class(class, visitor);
    }
    for enum_ in module.enum_.values_mut() {
        visit_enum(enum_, visitor);
    }
    for overloads in module.function.values_mut() {
        for function in overloads {
            visit_parameters(&mut function.parameters, visitor);
            visitor.type_info(&mut function.r#return);
        }
    }
    for variable in module.variables.values_mut() {
        visitor.type_info(&mut variable.type_);
    }
    for alias in module.type_aliases.values_mut() {
        visitor.type_info(&mut alias.type_);
    }
}

/// Visit every type in `class`, including its nested classes.
fn visit_class(class: &mut ClassDef, visitor: &mut impl Visitor) {
    for attr in &mut class.attrs {
        visit_member(attr, visitor);
    }
    for (getter, setter) in class.getter_setters.values_mut() {
        for member in [getter, setter].into_iter().flatten() {
            visit_member(member, visitor);
        }
    }
    for overloads in class.methods.values_mut() {
        for method in overloads {
            visit_method(method, visitor);
        }
    }
    for base in &mut class.bases {
        visitor.type_info(base);
    }
    for nested in &mut class.classes {
        visit_class(nested, visitor);
    }
}

/// Visit every type in `enum_`.
fn visit_enum(enum_: &mut EnumDef, visitor: &mut impl Visitor) {
    for method in &mut enum_.methods {
        visit_method(method, visitor);
    }
    for member in enum_
        .attrs
        .iter_mut()
        .chain(&mut enum_.getters)
        .chain(&mut enum_.setters)
    {
        visit_member(member, visitor);
    }
}

/// Visit the type of `member`.
fn visit_member(member: &mut MemberDef, visitor: &mut impl Visitor) {
    visitor.type_info(&mut member.r#type);
}

/// Visit the parameters and return type of `method`.
fn visit_method(method: &mut MethodDef, visitor: &mut impl Visitor) {
    visit_parameters(&mut method.parameters, visitor);
    visitor.type_info(&mut method.r#return);
}

/// Visit the types and defaults of `parameters`.
fn visit_parameters(parameters: &mut Parameters, visitor: &mut impl Visitor) {
    let Parameters {
        positional_only,
        positional_or_keyword,
        keyword_only,
        varargs,
        varkw,
    } = parameters;
    let all = positional_only
        .iter_mut()
        .chain(positional_or_keyword)
        .chain(keyword_only)
        .chain(varargs.as_mut())
        .chain(varkw.as_mut());
    for Parameter {
        type_info, default, ..
    } in all
    {
        visitor.type_info(type_info);
        visitor.default(default);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    /// The module classes are relocated from.
    const SOURCE: &str = "dep._dep.configuration";
    /// The module classes are relocated to.
    const TARGET: &str = "pkg._pkg.client";

    /// A hand-written type that imports `module`.
    fn type_info(name: &str, module: &str) -> TypeInfo {
        TypeInfo {
            name: name.to_string(),
            source_module: None,
            import: HashSet::from([ImportRef::Module(module.into())]),
            type_refs: HashMap::new(),
        }
    }

    /// A [`Rename`] of `moved` for types written in `in_module`.
    fn rename<'a>(moved: &'a BTreeSet<&'static str>, in_module: &'a str) -> Rename<'a> {
        Rename {
            source: SOURCE,
            to: TARGET,
            moved,
            in_module,
        }
    }

    #[test]
    fn rename_rewrites_hand_written_types() {
        let moved = BTreeSet::from(["OAuthSession", "AuthServer"]);
        let mut rename = rename(&moved, "pkg");

        let mut info = type_info(
            "typing.Optional[configuration.OAuthSession] | configuration.LoadError | myconfiguration.AuthServer",
            SOURCE,
        );
        rename.type_info(&mut info);
        assert_eq!(
            info.name,
            "typing.Optional[client.OAuthSession] | configuration.LoadError | myconfiguration.AuthServer"
        );
        // Still needed for `LoadError`.
        assert!(info.import.contains(&ImportRef::Module(SOURCE.into())));
        assert!(info.import.contains(&ImportRef::Module(TARGET.into())));

        let mut info = type_info("typing.Optional[configuration.OAuthSession]", SOURCE);
        rename.type_info(&mut info);
        assert_eq!(info.name, "typing.Optional[client.OAuthSession]");
        assert!(!info.import.contains(&ImportRef::Module(SOURCE.into())));

        // A different module that happens to be called `configuration`.
        let mut info = type_info("configuration.AuthServer", "pkg.configuration");
        let before = info.clone();
        rename.type_info(&mut info);
        assert_eq!(info, before);
    }

    #[test]
    fn rename_unqualifies_names_in_target_module() {
        let moved = BTreeSet::from(["OAuthSession"]);
        let mut info = type_info("typing.Optional[configuration.OAuthSession]", SOURCE);
        rename(&moved, TARGET).type_info(&mut info);
        assert_eq!(info.name, "typing.Optional[OAuthSession]");
    }

    #[test]
    fn rename_qualifies_bare_names_in_source_module() {
        let moved = BTreeSet::from(["AuthServer"]);

        let mut info = TypeInfo::unqualified("collections.abc.Callable[[AuthServer], str]");
        rename(&moved, SOURCE).type_info(&mut info);
        assert_eq!(
            info.name,
            "collections.abc.Callable[[client.AuthServer], str]"
        );
        assert!(info.import.contains(&ImportRef::Module(TARGET.into())));

        // Elsewhere, a bare name can't mean the source module's class.
        let mut info = TypeInfo::unqualified("AuthServer");
        rename(&moved, "pkg").type_info(&mut info);
        assert_eq!(info.name, "AuthServer");
    }

    #[test]
    fn map_paths_skips_numbers() {
        assert_eq!(
            map_paths("typing.Literal[1, X]", |path| Some(format!("<{path}>"))),
            "<typing.Literal>[1, <X>]"
        );
    }
}

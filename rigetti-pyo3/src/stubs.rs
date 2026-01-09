//! Provide support for sorting `.pyi` stubs.
//!
//! [`pyo3-stub-gen`] doesn't keep its output in a consistent order.
//! Thankfully the order is deterministic if the program is deterministic,
//! but due to the use of the [`inventory`][] crate, it's not deterministic if the code changes.
//! This module thus sorts the components of stubs that are not already in a fixed order.
//!
//! [`pyo3-stub-gen`]: https://github.com/Jij-Inc/pyo3-stub-gen
//!
//! # Example Usage
//!
//! ```rust
//! # fn stub_info() -> pyo3_stub_gen::Result<pyo3_stub_gen::generate::StubInfo> {
//! #   Ok(pyo3_stub_gen::generate::StubInfo{ modules: Default::default(), python_root: Default::default() })
//! # }
//! fn main() -> pyo3_stub_gen::Result<()> {
//!    let mut stub = stub_info()?; // see [`pyo3_stub_gen::generate::StubInfo`]
//!    rigetti_pyo3::stubs::sort(&mut stub);
//!    stub.generate()?;
//!    Ok(())
//! }
//! ```

use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
};

use indexmap::IndexMap;
use itertools::Itertools as _;
use pyo3_stub_gen::{
    generate::{
        ClassDef, EnumDef, MemberDef, MethodDef, MethodType, Module, Parameter, ParameterDefault,
        Parameters,
    },
    type_info::{DeprecatedInfo, IgnoreTarget, ParameterKind},
    StubInfo, TypeInfo,
};

/// Sort, in place, all the unsorted components of a [`StubInfo`].
///
/// See the module-level documentation for more information.
pub fn sort(stub: &mut StubInfo) {
    let StubInfo {
        modules,
        // The Python root is fixed and doesn't need any adjustment.
        python_root: _,
    } = stub;

    // The `modules` are sorted because they're in a `BTreeMap`,
    // but we need to sort their contents.
    <BTreeMap<String, Module>>::values_mut(modules).for_each(sort_module);
}

/// A trait that's equivalent to [`Ord`] but not semantically meaningful,
/// used for putting [`pyo3_stub_gen`] types in a consistent order.
trait ArbitraryOrd {
    /// Analogous to [`Ord::cmp`]
    #[must_use]
    fn cmp(&self, other: &Self) -> Ordering;
}

/// Wrap references to an [`Ord`] type as an [`ArbitraryOrd`] type.
struct Arbitrary<'a, T>(&'a T);

impl<T: ArbitraryOrd> PartialEq for Arbitrary<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: ArbitraryOrd> Eq for Arbitrary<'_, T> {}

impl<T: ArbitraryOrd> PartialOrd for Arbitrary<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ArbitraryOrd> Ord for Arbitrary<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(other.0)
    }
}

/// Implements [`ArbitraryOrd`] for structs with named fields.
macro_rules! arbitrary_ord_structs {
    ($(
        $struct:ident { $($field:ident),* $(,)? };
    )*) => {
        $(
            #[automatically_derived]
            impl ArbitraryOrd for $struct {
                fn cmp(&self, other: &Self) -> Ordering {
                    // This guarantees us exhaustiveness
                    let $struct { $($field),* } = self;

                    // Return the first non-Equal result when comparing field pairs,
                    // or return Equal if all field pairs compare Equal.
                    let result = Ordering::Equal;
                    $(
                        let result = $field.cmp(&other.$field);
                        if result != Ordering::Equal {
                            return result;
                        }
                    )*
                    return result;
                }
            }
        )*
    }
}

impl<T: Ord> ArbitraryOrd for HashSet<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_sorted: Vec<_> = self.iter().sorted().collect();
        let other_sorted: Vec<_> = other.iter().sorted().collect();
        self_sorted.cmp(&other_sorted)
    }
}

impl<T: ArbitraryOrd> ArbitraryOrd for Option<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(left), Some(right)) => left.cmp(right),
        }
    }
}

impl<T: ArbitraryOrd> ArbitraryOrd for (T, T) {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).then_with(|| self.1.cmp(&other.1))
    }
}

impl<T: ArbitraryOrd> ArbitraryOrd for &T {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self).cmp(*other)
    }
}

impl<T: ArbitraryOrd> ArbitraryOrd for Vec<T> {
    fn cmp<'a>(&'a self, other: &'a Self) -> Ordering {
        let sort = |vec: &'a Self| -> Vec<_> {
            vec.iter()
                .sorted_by(ArbitraryOrd::cmp)
                .map(Arbitrary)
                .collect()
        };

        sort(self).cmp(&sort(other))
    }
}

impl<K: Ord, V: ArbitraryOrd> ArbitraryOrd for IndexMap<K, V> {
    fn cmp<'a>(&'a self, other: &'a Self) -> Ordering {
        let sort = |map: &'a Self| -> Vec<_> {
            map.iter()
                .sorted_by(|(lk, _), (rk, _)| lk.cmp(rk))
                .map(|(k, v)| (k, Arbitrary(v)))
                .collect()
        };
        sort(self).cmp(&sort(other))
    }
}

impl ArbitraryOrd for IgnoreTarget {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::All, Self::All) => Ordering::Equal,
            (Self::All, Self::Specified(_)) => Ordering::Less,
            (Self::Specified(_), Self::All) => Ordering::Greater,
            (Self::Specified(left), Self::Specified(right)) => left.cmp(right),
        }
    }
}

impl ArbitraryOrd for MethodType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Instance, Self::Instance)
            | (Self::Static, Self::Static)
            | (Self::Class, Self::Class)
            | (Self::New, Self::New) => Ordering::Equal,
            (Self::Instance | Self::Static | Self::Class, _) => Ordering::Less,
            (_, Self::Instance | Self::Static | Self::Class) => Ordering::Greater,
        }
    }
}

impl ArbitraryOrd for ParameterKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::PositionalOnly, Self::PositionalOnly)
            | (Self::PositionalOrKeyword, Self::PositionalOrKeyword)
            | (Self::KeywordOnly, Self::KeywordOnly)
            | (Self::VarPositional, Self::VarPositional)
            | (Self::VarKeyword, Self::VarKeyword) => Ordering::Equal,
            (
                Self::PositionalOnly
                | Self::PositionalOrKeyword
                | Self::KeywordOnly
                | Self::VarPositional,
                _,
            ) => Ordering::Less,
            (
                _,
                Self::PositionalOrKeyword
                | Self::PositionalOnly
                | Self::KeywordOnly
                | Self::VarPositional,
            ) => Ordering::Greater,
        }
    }
}

impl ArbitraryOrd for ParameterDefault {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::None, Self::None) => Ordering::Equal,
            (Self::None, _) => Ordering::Less,
            (_, Self::None) => Ordering::Greater,
            (Self::Expr(x), Self::Expr(y)) => x.cmp(y),
        }
    }
}

arbitrary_ord_structs! {
    TypeInfo { name, import };
    MemberDef { name, r#type, doc, default, deprecated };
    MethodDef { name, parameters, r#return, doc, r#type, is_async, deprecated, type_ignored, is_overload };
    DeprecatedInfo { since, note };
    ClassDef { name, doc, attrs, getter_setters, methods, bases, classes, match_args, subclass };
    Parameter { name, kind, type_info, default };
    Parameters { positional_only, positional_or_keyword, keyword_only, varargs, varkw };
}

// Inside the sorting functions, we check *every field* to check if we should sort it.  In order
// to make sure we've covered everything, and to get the compiler to yell at us if we've missed
// anything or there are any changes, we aggressively over-annotate all the types.  This allows
// seeing immediately where sorting bottoms out.

/// A key function usable to sort `IndexMap<String, T>` by key without `clone`ing the `String`s.
fn cmp_strings<T>(k1: &String, _: &T, k2: &String, _: &T) -> Ordering {
    k1.cmp(k2)
}

/// Sort elements of a class definition.
fn sort_class(class: &mut ClassDef) {
    let ClassDef {
        name: _, // These strings don't need adjustment.
        doc: _,
        attrs: _, // Regardless of the type of the field, we can't reorder attributes.
        bases: _, // Regardless of the type of the field, we can't reorder base classes.
        classes,
        match_args: _, // Regardless of the type of the field, we can't reorder match args.
        subclass: _,
        methods, // A map from names to overload sets; overloads can't be reordered
        getter_setters,
    } = class;

    // [`MemberDef`]s are atomic and don't have contents that need to be sorted.
    methods.sort_by(cmp_strings);
    getter_setters.sort_by(cmp_strings);

    // Finally, [`ClassDef`]s both need to be sorted internally
    // and need to be produced in sorted order.
    classes.iter_mut().for_each(sort_class);
    classes.sort_by(ArbitraryOrd::cmp);
}

/// Sort elements of an enum definition.
fn sort_enum(r#enum: &mut EnumDef) {
    let EnumDef {
        name: _, // These strings don't need adjustment.
        doc: _,
        variants: _, // Regardless of the type of the field, we can't reorder the variants.
        methods,
        attrs: _, // Regardless of the type of the field, we can't reorder the variants attributes.
        getters,
        setters,
    } = r#enum;

    // [`MethodDef`]s and [`MemberDef`]s are atomic and don't have contents that need to be sorted.
    methods.sort_by(ArbitraryOrd::cmp);
    getters.sort_by(ArbitraryOrd::cmp);
    setters.sort_by(ArbitraryOrd::cmp);
}

/// Sort elements of a module definition.
fn sort_module(module: &mut Module) {
    let Module {
        name: _, // Strings need no adjustment.
        doc: _,
        default_module_name: _,
        class,
        enum_,
        // `function` is an ordered map from function names to overload sets;
        // since overload sets themselves can't be reordered, there's nothing else to do.
        function: _,
        // [`VariableDef`]s are atomic;
        // they don't themselves have internal structure that needs to be reordered.
        variables: _,
        // The submodules are stored as an ordered set of names,
        // so we don't need to do any more sorting.
        submodules: _,
    } = module;

    // The [`class`]es and [`enum_`]s are sorted because they're kept in [`BTreeMap`]s, but we
    // need to sort the individual [`ClassDef`]s and [`EnumDef`]s internally.  You might be,
    // rightly, concerned about the fact that the ordering on [`TypeId`]s is arbitrary; however,
    // [`pyo3_stub_gen`] sorts the [`ClassDef`]s and [`EnumDef`]s by their names before writing
    // them out.
    // <BTreeMap<TypeId, ClassDef>>::
    // <BTreeMap<TypeId, EnumDef>>::values_mut(enum_)
    class.values_mut().for_each(sort_class);
    enum_.values_mut().for_each(sort_enum);
}

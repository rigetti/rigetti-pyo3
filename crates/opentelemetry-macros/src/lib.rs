// Copyright 2023 Rigetti Computing
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

// Covers correctness, suspicious, style, complexity, and perf
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![warn(clippy::nursery)]
// Has false positives that conflict with unreachable_pub
#![allow(clippy::redundant_pub_crate)]
#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    bad_style,
    dead_code,
    keyword_idents,
    improper_ctypes,
    macro_use_extern_crate,
    meta_variable_misuse, // May have false positives
    missing_abi,
    missing_debug_implementations, // can affect compile time/code size
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    noop_method_call,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    semicolon_in_expressions_from_macros,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_parens,
    unused_qualifications,
    variant_size_differences,
    while_true
)]
//! This crate defines a macro for annotating `fn` and `impl` in order to wrap functions
//! or methods with an `OpenTelemetry` context from Python using the `pyo3` crate.
use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{spanned::Spanned, ToTokens};
use syn::{
    meta::ParseNestedMeta,
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr, Signature,
};

const ERROR_UNSUPPORTED_ERROR_HANDLER: &str =
    "error handlers must be one of py_error, trace, print, or ignore";

const ERROR_NO_ASYNC_SUPPORT: &str =
    "pypropagate does not support async functions because there is no current support in pyo3";

const ERROR_ONLY_FN_OR_IMPL: &str = "pypropagate can only be used on functions or impl blocks";

const ERROR_SIGNATURE_MUST_INCLUDE_PY: &str =
    "pypropagate requires a first function parameter of type `pyo3::Python<'_>`";

const ERROR_UNEXPECTED_RECEIVER: &str = "found unexpected receiver argument";

const ERROR_UNKNOWN_CONFIGURATION_OPTION: &str = "unknown configuration option";

const ERROR_INVALID_EXCLUDE: &str = "exclude should only contain impl method names";

#[derive(PartialEq, Debug)]
enum RuntimeErrorHandler {
    PyError,
    Trace,
    Print,
    Ignore,
}

impl Parse for RuntimeErrorHandler {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value = input.parse::<LitStr>()?;
        match value.value().as_str() {
            "py_error" => Ok(Self::PyError),
            "trace" => Ok(Self::Trace),
            "print" => Ok(Self::Print),
            "ignore" => Ok(Self::Ignore),
            _ => Err(syn::Error::new(
                value.span(),
                format!(
                    "{ERROR_UNSUPPORTED_ERROR_HANDLER}: {error_handler}",
                    error_handler = value.value(),
                ),
            )),
        }
    }
}

#[derive(PartialEq, Debug)]
struct Configuration {
    on_context_extraction_failure: RuntimeErrorHandler,
    context_guard_name: String,
    exclude: HashSet<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            on_context_extraction_failure: RuntimeErrorHandler::Print,
            context_guard_name: "_pyo3_opentelemetry_context_guard".to_string(),
            exclude: HashSet::new(),
        }
    }
}

impl Configuration {
    fn add_nested_meta_item_impl(
        &mut self,
        meta: &ParseNestedMeta,
        valid_exclusions: &HashSet<String>,
    ) -> syn::Result<()> {
        if meta.path.is_ident("exclude") {
            let mut exclude = HashSet::new();
            meta.parse_nested_meta(|nested_meta| {
                let fn_name = nested_meta
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new(
                            nested_meta.path.__span(),
                            format!(
                                "{ERROR_INVALID_EXCLUDE}: {}",
                                nested_meta
                                    .path
                                    .to_token_stream()
                                    .to_string()
                                    .replace(" :: ", "::")
                            ),
                        )
                    })?
                    .to_string();
                if !valid_exclusions.contains(&fn_name) {
                    return Err(syn::Error::new(
                        nested_meta.path.__span(),
                        format!("{ERROR_INVALID_EXCLUDE}: {fn_name}"),
                    ));
                }
                exclude.insert(fn_name);
                Ok(())
            })?;
            self.exclude = exclude;
            Ok(())
        } else {
            self.add_nested_meta(meta)
        }
    }

    fn add_nested_meta_item_fn(&mut self, meta: &ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("exclude") {
            Err(syn::Error::new(
                meta.path.__span(),
                format!("{ERROR_INVALID_EXCLUDE}: configured function"),
            ))
        } else {
            self.add_nested_meta(meta)
        }
    }

    fn add_nested_meta(&mut self, meta: &ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("on_context_extraction_failure") {
            self.on_context_extraction_failure = meta.value()?.parse()?;
        } else if meta.path.is_ident("context_guard_name") {
            let value: LitStr = meta.value()?.parse()?;
            self.context_guard_name = value.value();
        } else {
            return Err(syn::Error::new(
                meta.path.__span(),
                format!(
                    "{ERROR_UNKNOWN_CONFIGURATION_OPTION}: {option}",
                    option = meta.path.to_token_stream(),
                ),
            ));
        }
        Ok(())
    }
}

fn wrap_block_in_current_context(
    block: &mut syn::Block,
    python_name: &proc_macro2::TokenStream,
    config: &Configuration,
) -> Result<(), proc_macro2::LexError> {
    let body = block.to_token_stream();
    let context_guard_name: proc_macro2::TokenStream = config.context_guard_name.parse()?;
    let error_handler: proc_macro2::TokenStream = match config.on_context_extraction_failure {
        RuntimeErrorHandler::Trace => syn::parse_quote! {
            if let Err(e) = #context_guard_name {
                use ::pyo3_opentelemetry::__opentelemetry::trace::TraceContextExt;
                let ctx = ::pyo3_opentelemetry::__opentelemetry::Context::current();
                ctx.span().record_error(&e);
            }
        },
        RuntimeErrorHandler::PyError => syn::parse_quote! {
            let #context_guard_name = #context_guard_name?;
        },
        RuntimeErrorHandler::Print => syn::parse_quote! {
            if let Err(e) = #context_guard_name {
                eprintln!("{}", e);
            }
        },
        RuntimeErrorHandler::Ignore => syn::parse_quote! {
            let #context_guard_name = #context_guard_name.ok();
        },
    };
    let current_context_setter = syn::parse_quote! {
        {
            let #context_guard_name  = pyo3_opentelemetry::attach_otel_context_from_python(#python_name);
            #error_handler
            #body
        }
    };
    *block = current_context_setter;
    Ok(())
}

fn validate_signature(signature: &Signature) -> syn::Result<()> {
    if signature.asyncness.is_some() {
        return Err(syn::Error::new(
            signature.asyncness.__span(),
            ERROR_NO_ASYNC_SUPPORT,
        ));
    }
    Ok(())
}

fn get_python_parameter_name(signature: &Signature) -> syn::Result<proc_macro2::TokenStream> {
    let mut first_arg = signature
        .inputs
        .first()
        .ok_or_else(|| syn::Error::new(signature.__span(), ERROR_SIGNATURE_MUST_INCLUDE_PY))?;
    if let syn::FnArg::Receiver(_) = first_arg {
        first_arg =
            signature.inputs.iter().nth(1).ok_or_else(|| {
                syn::Error::new(signature.__span(), ERROR_SIGNATURE_MUST_INCLUDE_PY)
            })?;
    }

    match first_arg {
        syn::FnArg::Typed(arg) => Ok(arg.pat.to_token_stream()),
        syn::FnArg::Receiver(_) => Err(syn::Error::new(
            first_arg.__span(),
            ERROR_UNEXPECTED_RECEIVER,
        )),
    }
}

fn pypropagate_signature_and_method(
    signature: &Signature,
    block: &mut syn::Block,
    config: &Configuration,
) -> Result<(), syn::Error> {
    validate_signature(signature)?;
    let python_name = get_python_parameter_name(signature)?;

    wrap_block_in_current_context(block, &python_name, config)?;
    Ok(())
}

fn pypropagate_impl(item: syn::Item, config: &Configuration) -> Result<syn::Item, syn::Error> {
    match item {
        syn::Item::Fn(mut item_fn) => {
            pypropagate_signature_and_method(&item_fn.sig, &mut item_fn.block, config)?;

            Ok(syn::Item::Fn(item_fn))
        }
        syn::Item::Impl(mut item_impl) => {
            for mut item in &mut item_impl.items {
                if let syn::ImplItem::Fn(ref mut item_method) = &mut item {
                    if config.exclude.contains(&item_method.sig.ident.to_string()) {
                        continue;
                    }
                    pypropagate_signature_and_method(
                        &item_method.sig,
                        &mut item_method.block,
                        config,
                    )?;
                }
            }
            Ok(syn::Item::Impl(item_impl))
        }
        _ => Err(syn::Error::new_spanned(item, ERROR_ONLY_FN_OR_IMPL)),
    }
}

fn get_item_impl_method_names(item_impl: &syn::ItemImpl) -> HashSet<String> {
    let mut valid_exclusions: HashSet<String> = HashSet::new();
    for item in &item_impl.items {
        if let syn::ImplItem::Fn(item_method) = item {
            valid_exclusions.insert(item_method.sig.ident.to_string());
        }
    }
    valid_exclusions
}

/// Ensure the wrapped function or method executes from Python in the current OpenTelemetry context.
///
/// This macro prepends `pyo3_opentelemetry::attach_otel_context_from_python` to the function or method body,
/// effectively ensuring that the wrapped function or method is executed in the current `OpenTelemetry` context
/// from the Python side.
///
/// # Requirements and Limitations
///
/// * The first non-receiver parameter MUST be `pyo3::Python`.
/// * The macro MUST be invoked on the outside of `pyfunction` or `pymethods`.
/// * The calling Python code must have [opentelemetry-api](https://pypi.org/project/opentelemetry-api/) installed.
/// * `async` function signatures are not supported as they are not supported in `pyo3`.
///
/// # Configuration
///
/// The macro can be configured with the following attributes:
/// - `context_guard_name`: The name of the variable that will be used to guard the context. Defaults to `_pyo3_opentelemetry_context_guard`.
///   - must be a valid Rust identifier.
/// - `on_context_extraction_failure`: What to do when the context cannot be extracted from Python. Defaults to `print`.
///   - `print`: Print the error to stderr and continue.
///   - `trace`: Record the error on the current span using `opentelemetry::trace::TraceContextExt::record_error`.
///   - `py_error`: Return a `pyo3::PyErr`.
///   - `ignore`: Ignore the error.
/// - `exclude`: A list of method names to exclude. Only valid on `impl` items.
///
/// # Examples
///
/// ```ignore
/// use pyo3::prelude::*;
/// use pyo3_opentelemetry::pypropagate;
///
/// #[pypropagate]
/// #[pyfunction]
/// fn my_function1(py: Python<'_>, arg1: u32, arg2: String) -> PyResult<()> {
///    // ...
///    Ok(())
/// }
///
/// #[pypropagate(context_guard_name = "_my_context_guard", on_context_extraction_failure = "py_error")]
/// #[pyfunction]
/// fn my_function2(py: Python<'_>, arg1: u32, arg2: String) -> PyResult<()> {
///    // ...
///    Ok(())
/// }
///
/// struct MyType;
///
/// #[pypropagate]
/// #[pymethods(exclude(new))]
/// impl MyType {
///    #[new]
///    fn new() -> Self {
///       Self
///    }
///
///    fn my_method(&self, py: Python<'_>, arg1: u32, arg2: String) -> PyResult<()> {
///      // ...
///      Ok(())
///    }
/// }
/// ```
#[proc_macro_attribute]
pub fn pypropagate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);

    // https://docs.rs/syn/latest/syn/struct.Attribute.html#method.parse_nested_meta

    let mut config = Configuration::default();
    if !attr.is_empty() {
        if let syn::Item::Impl(item_impl) = &item {
            let valid_exclusions: HashSet<String> = get_item_impl_method_names(item_impl);
            let config_parser = syn::meta::parser(|meta| {
                config.add_nested_meta_item_impl(&meta, &valid_exclusions)
            });
            parse_macro_input!(attr with config_parser);
        } else {
            let config_parser = syn::meta::parser(|meta| config.add_nested_meta_item_fn(&meta));
            parse_macro_input!(attr with config_parser);
        }
    }

    pypropagate_impl(item, &config).map_or_else(
        |e| e.to_compile_error().into(),
        |item| item.to_token_stream().into(),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
    use syn::parse_quote;

    /// Test that `pypropagate` nested meta is properly parsed to a `Configuration`.
    #[rstest]
    #[case("#[pypropagate]", Configuration::default())]
    #[case("#[pypropagate(on_context_extraction_failure = \"py_error\")]", Configuration{on_context_extraction_failure: RuntimeErrorHandler::PyError, ..Default::default()})]
    #[case("#[pypropagate(on_context_extraction_failure = \"trace\")]", Configuration { on_context_extraction_failure: RuntimeErrorHandler::Trace, ..Default::default() })]
    #[case("#[pypropagate(on_context_extraction_failure = \"print\")]", Configuration { on_context_extraction_failure: RuntimeErrorHandler::Print, ..Default::default() })]
    #[case("#[pypropagate(on_context_extraction_failure = \"ignore\")]", Configuration { on_context_extraction_failure: RuntimeErrorHandler::Ignore, ..Default::default() })]
    #[case("#[pypropagate(on_context_extraction_failure = \"print\", context_guard_name = \"_my_guard\")]", Configuration { on_context_extraction_failure: RuntimeErrorHandler::Print, context_guard_name: "_my_guard".to_string(), ..Default::default() })]
    #[case(
        "#[pypropagate(context_guard_name = \"_my_guard\")]",
        Configuration { context_guard_name: "_my_guard".to_string(), ..Configuration::default() }
    )]
    #[case(
        "#[pypropagate(exclude(my_method))]",
        Configuration { exclude: HashSet::from(["my_method".to_string()]), ..Configuration::default() }
    )]
    fn test_configuration_parsing(#[case] attr: &str, #[case] expected: Configuration) {
        let mut config = Configuration::default();
        let tokens: proc_macro2::TokenStream = syn::parse_str(attr).unwrap();
        let attr: syn::Attribute = parse_quote! {
            #tokens
        };

        match attr.meta {
            syn::Meta::Path(_) => {}
            _ => {
                attr.parse_nested_meta(|meta| {
                    config
                        .add_nested_meta_item_impl(&meta, &HashSet::from(["my_method".to_string()]))
                })
                .unwrap();
            }
        }

        assert_eq!(config, expected);
    }

    /// Test validation of nested meta on the `pypropagate` macro.
    #[rstest]
    #[case("#[pypropagate(not_an_option = \"\")]", [ERROR_UNKNOWN_CONFIGURATION_OPTION, ": not_an_option"])]
    #[case("#[pypropagate(on_context_extraction_failure = \"not_py_error\")]", [ERROR_UNSUPPORTED_ERROR_HANDLER, ": not_py_error"])]
    #[case("#[pypropagate(exclude(not::an::ident))]", [ERROR_INVALID_EXCLUDE, ": not::an::ident"])]
    fn test_misconfiguration_errors(#[case] attr: &str, #[case] expected_error: [&str; 2]) {
        let mut config = Configuration::default();
        let tokens: proc_macro2::TokenStream = syn::parse_str(attr).unwrap();
        let attr: syn::Attribute = parse_quote! {
            #tokens
        };
        let assumed_method_names: HashSet<String> = HashSet::new();
        let result = attr.parse_nested_meta(|meta| {
            config.add_nested_meta_item_impl(&meta, &assumed_method_names)
        });
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), expected_error.join(""));
    }

    const MISSING_PY_FN_PARAMETER: &str = r"
        fn my_function() {}
    ";

    const MISSING_PY_IMPL_PARAMETER: &str = r"
        impl MyType {
            fn my_method(&self) {}
        }
    ";

    const ASYNC_FUNCTION: &str = r"
        async fn my_function() {}
    ";

    const ASYNC_METHOD: &str = r"
        impl MyType {
            async fn my_method(&self) {}
        }";

    const STRUCT: &str = r"
        struct MyStruct;
        ";

    /// Test that invalid macro invocations result in the correct error messages.
    #[rstest]
    #[case(MISSING_PY_FN_PARAMETER, ERROR_SIGNATURE_MUST_INCLUDE_PY)]
    #[case(MISSING_PY_IMPL_PARAMETER, ERROR_SIGNATURE_MUST_INCLUDE_PY)]
    #[case(ASYNC_FUNCTION, ERROR_NO_ASYNC_SUPPORT)]
    #[case(ASYNC_METHOD, ERROR_NO_ASYNC_SUPPORT)]
    #[case(STRUCT, ERROR_ONLY_FN_OR_IMPL)]
    fn test_pypropagate_macro_errors(#[case] code: &str, #[case] error: &str) {
        let tokens: proc_macro2::TokenStream = syn::parse_str(code).unwrap();
        let item = syn::parse2::<syn::Item>(tokens).unwrap();
        let config = Configuration::default();
        let result = pypropagate_impl(item, &config)
            .map(|item| item.to_token_stream())
            .unwrap_err();
        assert_eq!(result.to_string(), error);
    }

    /// Test that configuration of "exclude" is properly validated on methods and functions.
    #[rstest]
    #[case(VALID_METHODS, "#[pypropagate(exclude(doesnt_exist))]", "doesnt_exist")]
    #[case(
        VALID_FUNCTION,
        "#[pypropagate(exclude(my_function))]",
        "configured function"
    )]
    fn test_invalid_exclusions(
        #[case] code: &str,
        #[case] attr: &str,
        #[case] expected_error: &str,
    ) {
        let mut config = Configuration::default();
        let tokens: proc_macro2::TokenStream = syn::parse_str(attr).unwrap();
        let attr: syn::Attribute = parse_quote! {
            #tokens
        };
        let tokens: proc_macro2::TokenStream = syn::parse_str(code).unwrap();
        let item = syn::parse2::<syn::Item>(tokens).unwrap();
        let result = match item {
            syn::Item::Impl(item_impl) => {
                let valid_exclusions = get_item_impl_method_names(&item_impl);
                attr.parse_nested_meta(|meta| {
                    config.add_nested_meta_item_impl(&meta, &valid_exclusions)
                })
            }
            syn::Item::Fn(_) => {
                attr.parse_nested_meta(|meta| config.add_nested_meta_item_fn(&meta))
            }
            _ => panic!("Invalid item type"),
        };

        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("{ERROR_INVALID_EXCLUDE}: {expected_error}")
        );
    }

    const VALID_FUNCTION: &str = r"
        fn my_function(py: Python<'_>) {}
        ";

    const VALID_METHODS: &str = r"
        impl MyType {
            fn my_method1(&self) {}

            fn my_method2(&self, py: Python<'_>) {}
        }";

    /// Test that valid macro invocations do not result in errors.
    #[rstest]
    #[case(VALID_FUNCTION, Configuration::default())]
    #[case(VALID_METHODS, Configuration{ exclude: HashSet::from(["my_method1".to_string()]), ..Default::default() })]
    fn test_valid(#[case] code: &str, #[case] config: Configuration) {
        let tokens: proc_macro2::TokenStream = syn::parse_str(code).unwrap();
        let item = syn::parse2::<syn::Item>(tokens).unwrap();
        let _result = pypropagate_impl(item, &config)
            .map(|item| item.to_token_stream())
            .expect("Should not fail");
    }
}

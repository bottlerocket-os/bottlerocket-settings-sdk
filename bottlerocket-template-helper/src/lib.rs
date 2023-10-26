//! This crate provides a procedural macro for defining template helpers in settings extensions.
//! See the documentation in [`bottlerocket-settings-sdk::helper`] for more information.
use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, FnArg, ItemFn};

#[derive(FromMeta)]
struct MacroArgs {
    ident: syn::Ident,
    vis: Option<String>,
}

/// Defines a [`bottlerocket-settings-sdk::helper::HelperDef`] based on a given function.
///
/// This macro requires:
/// * Your function arguments implement [`serde::Deserialize`]
/// * Your return value is a `Result<T, E>` where `T` implements [`serde::Serialize`]
///   and `E` implements `Into<Box<dyn std::error::Error>>`.
///
/// To define a `HelperDef` called `my_helper` based on a function, you could do something like:
///
/// ```
/// use bottlerocket_settings_sdk::helper::{HelperDef, template_helper};
///
/// #[template_helper(ident = my_helper)]
/// fn help_with(list_of_things: Vec<String>) -> Result<Vec<String>, anyhow::Error> {
///     Ok(list_of_things
///         .into_iter()
///         .map(|s| format!("Helped with '{s}'!"))
///         .collect())
/// }
/// ```
#[proc_macro_attribute]
pub fn template_helper(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: MacroArgs =
        MacroArgs::from_list(&NestedMeta::parse_meta_list(args.into()).unwrap()).unwrap();

    let helper_fn_name = args.ident;

    let fn_ast: ItemFn = syn::parse2(input.into()).unwrap();
    let fn_name = fn_ast.sig.ident.clone();

    let num_args = fn_ast.sig.inputs.len();
    let arg_types: Vec<Box<syn::Type>> = fn_ast
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => {
                panic!("template_helper macro does not work on methods that take `self`")
            }
            FnArg::Typed(t) => t.ty.clone(),
        })
        .collect();

    let mut helper_fn: ItemFn = syn::parse2(quote! {
        fn #helper_fn_name(
            args: Vec<serde_json::Value>,
        ) -> std::result::Result<
            serde_json::Value,
            bottlerocket_settings_sdk::HelperError
        > {
            if args.len() != #num_args {
                return Err(bottlerocket_settings_sdk::HelperError::Arity {
                    expected_args: #num_args,
                    provided_args: args.len(),
                });
            }

            // Call the input function with our dynamically generated list of arguments.
            // We know that `args` is the correct length because we checked above, so we can let
            // the macro unwrap values that it takes.
            let mut args = args.into_iter();
            #fn_name(#(
                    {
                        let arg: #arg_types = match serde_json::from_value(args.next().unwrap()) {
                            Ok(parsed) => parsed,
                            Err(e) => return Err(bottlerocket_settings_sdk::HelperError::JSONParse { source: e })
                        };
                        arg
                    }
                ),*)
                .map_err(|e| bottlerocket_settings_sdk::HelperError::HelperExecute {
                    source: e.into(),
                })
                .and_then(|result| serde_json::to_value(result).map_err(|e| {
                    bottlerocket_settings_sdk::HelperError::JSONSerialize { source: e }
                }))
        }
    })
    .unwrap();

    if let Some(visibility) = args.vis {
        helper_fn.vis = syn::parse_str(&visibility).unwrap();
    }

    quote! {
        #fn_ast

        #helper_fn
    }
    .into()
}

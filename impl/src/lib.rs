mod args;
mod decl;
mod imp;
mod utils;

use args::{DeclArgs, ImplArgs, ReprType};
use proc_macro::TokenStream;
use syn::{Error, parse_macro_input};

#[proc_macro_attribute]
pub fn extern_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    let x = if args.is_empty() {
        imp::expand(
            ImplArgs {
                extern_trait: syn::parse_quote!(::extern_trait),
                repr_type: ReprType::default(),
            },
            parse_macro_input!(input),
        )
    } else {
        // Try to parse as DeclArgs first (for trait declarations)
        // If that fails with a trait-specific error, try ImplArgs (for impl blocks with crate = ...)
        let args_clone = args.clone();
        match syn::parse::<DeclArgs>(args) {
            Ok(decl_args) => decl::expand(decl_args, parse_macro_input!(input)),
            Err(_) => {
                // Try parsing as ImplArgs instead
                imp::expand(parse_macro_input!(args_clone), parse_macro_input!(input))
            }
        }
    }
    .unwrap_or_else(Error::into_compile_error)
    .into();
    // panic!("{x}");
    x
}

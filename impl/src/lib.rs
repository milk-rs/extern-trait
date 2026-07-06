mod args;
mod decl;
mod imp;

use args::{Args, ImplArgs, TraitArgs};
use proc_macro::TokenStream;
use syn::{Error, Item, Result};

#[proc_macro_attribute]
pub fn extern_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    expand(args, input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(args: TokenStream, input: TokenStream) -> Result<proc_macro2::TokenStream> {
    let args = syn::parse::<Args>(args)?;

    match syn::parse::<Item>(input)? {
        Item::Trait(input) => decl::expand(TraitArgs::try_from(args)?, input),
        Item::Impl(input) => imp::expand(ImplArgs::try_from(args)?, input),
        input => Err(Error::new_spanned(
            input,
            "#[extern_trait] can only be used on a trait or trait impl",
        )),
    }
}

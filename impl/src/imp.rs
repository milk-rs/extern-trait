use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{ItemImpl, Result, spanned::Spanned};

use crate::args::ImplArgs;

pub fn expand(args: ImplArgs, input: ItemImpl) -> Result<TokenStream> {
    let Some((_, trait_, _)) = &input.trait_ else {
        return Err(syn::Error::new(Span::call_site(), "expected a trait impl"));
    };

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "#[extern_trait] impls may not have generics",
        ));
    }

    if let Some(where_clause) = &input.generics.where_clause {
        return Err(syn::Error::new_spanned(
            where_clause,
            "#[extern_trait] impls may not have a where clause",
        ));
    }

    let _ = args.extern_trait;
    let repr_type = &args.repr_type;
    let ptr_count = args.repr_type.ptr_count();
    let ptr_count_desc = ptr_count.to_string();
    let ty = &input.self_ty;

    let assert = quote_spanned! {ty.span()=>
        const _: () = {
            assert!(
                ::core::mem::size_of::<#ty>() <= ::core::mem::size_of::<#repr_type>(),
                concat!(
                    stringify!(#ty),
                    " is too large to be used with #[extern_trait] where ptr_count=",
                    #ptr_count_desc,
                )
            );
        };
    };

    Ok(quote! {
        #input

        #assert

        #trait_!(#trait_: #ty, ptr_count = #ptr_count);
    })
}

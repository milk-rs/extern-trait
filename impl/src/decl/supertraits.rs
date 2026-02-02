use std::{cell::LazyCell, collections::BTreeMap};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Path, PathArguments, PathSegment, Signature, TraitItemFn, parse_quote};

use super::{sig::VerifiedSignature, sym::Symbol};

#[derive(Debug, Clone)]
struct SuperTraitInfo {
    is_unsafe: bool,
    name: Ident,
    generics: usize,
    functions: Vec<VerifiedSignature>,
}

macro_rules! supertrait {
    (
        $is_unsafe:literal $name:ident $(<$gen:literal>)? {
            $($f:stmt)*
        }
    ) => {
        SuperTraitInfo {
            is_unsafe: $is_unsafe,
            name: parse_quote!($name),
            generics: 0 $(+ $gen)?,
            functions: vec![
                $({
                    let item: TraitItemFn = parse_quote!($f);
                    VerifiedSignature::try_new(&item.sig).unwrap()
                },)*
            ],
        }
    };
    (
        unsafe $($rest:tt)*
    ) => {
        supertrait!(true $($rest)*)
    };
    (
        $($rest:tt)*
    ) => {
        supertrait!(false $($rest)*)
    };
}

#[allow(clippy::declare_interior_mutable_const)]
const TRAITS: LazyCell<Vec<SuperTraitInfo>> = LazyCell::new(|| {
    vec![
        supertrait! { unsafe Send {} },
        supertrait! { unsafe Sync {} },
        supertrait! { Sized {} },
        supertrait! { Unpin {} },
        supertrait! { Copy {} },
        supertrait! { Debug {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result;
        } },
        supertrait! { Clone {
            fn clone(&self) -> Self;
        } },
        supertrait! { Default {
            fn default() -> Self;
        } },
        supertrait! { AsRef<1> {
            fn as_ref(&self) -> &____0;
        } },
        supertrait! { AsMut<1> {
            fn as_mut(&mut self) -> &mut ____0;
        } },
    ]
});

pub fn generate_impl(
    extern_trait: &Path,
    path: &PathSegment,
    proxy_ident: &Ident,
    sym: &Symbol,
) -> Option<(TokenStream, TokenStream)> {
    let PathSegment { ident, arguments } = path;

    #[allow(clippy::borrow_interior_mutable_const)]
    let t = TRAITS
        .iter()
        .find(|&t| {
            &t.name == ident
                && match (&arguments, t.generics) {
                    (PathArguments::None, 0) => true,
                    (PathArguments::AngleBracketed(args), n) => args.args.len() == n,
                    _ => false,
                }
        })
        .cloned()?;

    let unsafety = if t.is_unsafe {
        quote! { unsafe }
    } else {
        quote! {}
    };

    let mut replace_map = BTreeMap::new();
    if let PathArguments::AngleBracketed(args) = arguments {
        for (i, arg) in args.args.iter().enumerate() {
            replace_map.insert(format!("____{}", i), arg.to_token_stream().to_string());
        }
    }

    let transformed = t
        .functions
        .into_iter()
        .map(|sig| {
            let sig = sig.to_token_stream().to_string();
            let sig = replace_map
                .iter()
                .fold(sig, |acc, (k, v)| acc.replace(k, v));
            VerifiedSignature::try_new(&syn::parse_str::<Signature>(&sig).unwrap()).unwrap()
        })
        .collect::<Vec<_>>();

    let impl_content = transformed.iter().map(|sig| {
        let export_name = format!(
            "{:?}",
            sym.clone()
                .with_name(format!("{}::{}", path.to_token_stream(), sig.ident))
        );
        super::generate_proxy_impl(proxy_ident, &export_name, sig)
    });
    let macro_content = transformed.iter().map(|sig| {
        let export_name = format!(
            "{:?}",
            sym.clone()
                .with_name(format!("{}::{}", path.to_token_stream(), sig.ident))
        );
        super::generate_macro_rules(extern_trait, Some(quote!(#path)), &export_name, sig)
    });

    Some((
        quote! {
            #unsafety impl #path for #proxy_ident {
                #(#impl_content)*
            }
        },
        quote! {
            #(#macro_content)*
        },
    ))
}

use std::cell::LazyCell;

use quote::ToTokens;
use syn::{
    Ident, Path, PathArguments, PathSegment, Signature, Token, TraitBoundModifier, TraitItemFn,
    TypeParamBound, parse_quote, punctuated::Punctuated,
};

use super::types::VerifiedSignature;

#[derive(Debug, Clone)]
struct Supertrait {
    is_unsafe: bool,
    name: Ident,
    generics: usize,
    functions: Vec<VerifiedSignature>,
}

macro_rules! supertrait {
    (
        is_unsafe: $is_unsafe:expr,
        name: $name:ident $(<$gen:literal>)? {
            $($f:stmt)*
        }
    ) => {
        Supertrait {
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
        unsafe $name:ident {
            $($f:stmt)*
        }
    ) => {
        supertrait! {
            is_unsafe: true,
            name: $name { $($f)* }
        }
    };
    (
        $name:ident {
            $($f:stmt)*
        }
    ) => {
        supertrait! {
            is_unsafe: false,
            name: $name { $($f)* }
        }
    };
    (
        $name:ident <$gen:literal> {
            $($f:stmt)*
        }
    ) => {
        supertrait! {
            is_unsafe: false,
            name: $name <$gen> { $($f)* }
        }
    };
}

#[allow(clippy::declare_interior_mutable_const)]
const SUPERTRAITS: LazyCell<Vec<Supertrait>> = LazyCell::new(|| {
    vec![
        supertrait! { unsafe Send {} },
        supertrait! { unsafe Sync {} },
        supertrait! { Sized {} },
        supertrait! { Unpin {} },
        supertrait! { Copy {} },
        supertrait! { Eq {} },
        supertrait! { UnwindSafe {} },
        supertrait! { RefUnwindSafe {} },
        supertrait! { Freeze {} },
        supertrait! { Debug {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result;
        } },
        supertrait! { Display {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result;
        } },
        supertrait! { Clone {
            fn clone(&self) -> Self;
        } },
        supertrait! { Default {
            fn default() -> Self;
        } },
        supertrait! { PartialEq {
            fn eq(&self, other: &Self) -> bool;
        } },
        supertrait! { PartialOrd {
            fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering>;
        } },
        supertrait! { Ord {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering;
        } },
        supertrait! { AsRef<1> {
            fn as_ref(&self) -> &____0;
        } },
        supertrait! { AsMut<1> {
            fn as_mut(&mut self) -> &mut ____0;
        } },
        supertrait! { Borrow<1> {
            fn borrow(&self) -> &____0;
        } },
        supertrait! { BorrowMut<1> {
            fn borrow_mut(&mut self) -> &mut ____0;
        } },
    ]
});

#[derive(Debug, Clone)]
pub struct SupertraitInfo {
    pub is_unsafe: bool,
    pub path: Path,
    pub methods: Vec<VerifiedSignature>,
}

fn match_supertrait(path: &Path) -> Option<SupertraitInfo> {
    if path.leading_colon.is_some() || path.segments.len() != 1 {
        return None;
    }
    let PathSegment { ident, arguments } = &path.segments[0];

    #[allow(clippy::borrow_interior_mutable_const)]
    let t = SUPERTRAITS
        .iter()
        .find(|t| {
            &t.name == ident
                && match (&arguments, t.generics) {
                    (PathArguments::None, 0) => true,
                    (PathArguments::AngleBracketed(args), n) => args.args.len() == n,
                    _ => false,
                }
        })
        .cloned()?;

    let mut replace_map: Vec<(String, String)> = Vec::new();
    if let PathArguments::AngleBracketed(args) = &arguments {
        for (i, arg) in args.args.iter().enumerate() {
            replace_map.push((format!("____{}", i), arg.to_token_stream().to_string()));
        }
    }

    let methods = t
        .functions
        .iter()
        .map(|sig| {
            let sig_str = sig.to_token_stream().to_string();
            let sig_str = replace_map
                .iter()
                .fold(sig_str, |acc, (k, v)| acc.replace(k, v));
            let parsed = syn::parse_str::<Signature>(&sig_str).unwrap();
            VerifiedSignature::try_new(&parsed).unwrap()
        })
        .collect::<Vec<_>>();

    Some(SupertraitInfo {
        is_unsafe: t.is_unsafe,
        path: path.clone(),
        methods,
    })
}

pub fn collect_supertraits(
    supertraits: &Punctuated<TypeParamBound, Token![+]>,
) -> Vec<SupertraitInfo> {
    supertraits
        .iter()
        .filter_map(|bound| {
            if let TypeParamBound::Trait(t) = bound
                && matches!(t.modifier, TraitBoundModifier::None)
                && t.lifetimes.is_none()
                && let Some(info) = match_supertrait(&t.path)
            {
                Some(info)
            } else {
                None
            }
        })
        .collect()
}

mod supertraits;
mod symbol;
mod types;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Error, Ident, ItemTrait, Path, Result, ReturnType, TraitItem, Type, parse_quote};

use self::{
    supertraits::{SupertraitInfo, collect_supertraits},
    symbol::Symbol,
    types::VerifiedSignature,
};
use crate::args::DeclArgs;

pub fn expand(args: DeclArgs, input: ItemTrait) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            input.generics,
            "#[extern_trait] may not have generics",
        ));
    }

    let extern_trait = args.extern_trait;
    let proxy = args.proxy;

    let proxy_ident = &proxy.ident;
    let proxy = proxy.expand(&extern_trait);

    let vis = &input.vis;
    let unsafety = &input.unsafety;
    let trait_ident = &input.ident;

    let sym = Symbol::new(trait_ident.to_string());

    let trait_methods = collect_trait_methods(&input)?;
    let supertrait_infos = collect_supertraits(&input.supertraits);

    let mut trait_impl = TokenStream::new();
    let mut supertrait_impls = TokenStream::new();
    let mut macro_exports = TokenStream::new();

    for method in &trait_methods {
        let export_name = format!("{:?}", sym.clone().with_name(method.ident.to_string()));
        trait_impl.extend(generate_impl_method(proxy_ident, &export_name, method));
        macro_exports.extend(generate_macro_export(
            &extern_trait,
            None,
            &export_name,
            method,
        ));
    }

    for info in &supertrait_infos {
        let mut supertrait_impl = TokenStream::new();

        let SupertraitInfo {
            is_unsafe,
            path,
            methods,
        } = info;

        for method in methods {
            let export_name = format!(
                "{:?}",
                sym.clone()
                    .with_name(format!("{}::{}", path.to_token_stream(), method.ident))
            );
            supertrait_impl.extend(generate_impl_method(proxy_ident, &export_name, method));
            macro_exports.extend(generate_macro_export(
                &extern_trait,
                Some(path),
                &export_name,
                method,
            ));
        }

        let unsafety = is_unsafe.then(|| quote! { unsafe });

        supertrait_impls.extend(quote! {
            #unsafety impl #path for #proxy_ident {
                #supertrait_impl
            }
        });
    }

    let macro_ident = format_ident!("__extern_trait_{}", trait_ident);

    let drop_name = format!("{:?}", sym.clone().with_name("drop"));
    let typeid_name = format!("{:?}", sym.clone().with_name("typeid"));
    let panic_doc = format!(
        "# Panics\nPanics if the type parameter `T` is not an implementation type for \
         #[extern_trait] `{}`.",
        trait_ident
    );

    Ok(quote! {
        #input

        #proxy

        #unsafety impl #trait_ident for #proxy_ident {
            #trait_impl
        }

        #supertrait_impls

        impl Drop for #proxy_ident {
            fn drop(&mut self) {
                unsafe extern "Rust" {
                    #[link_name = #drop_name]
                    unsafe fn drop(this: *mut #proxy_ident);
                }
                unsafe { drop(self) }
            }
        }

        impl #proxy_ident {
            fn assert_type_is_impl<T: #trait_ident>() {
                unsafe extern "Rust" {
                    #[link_name = #typeid_name]
                    safe static TYPEID: #extern_trait::__private::ConstTypeId;
                }
                let typeid = #extern_trait::__private::ConstTypeId::of::<T>();
                assert!(
                    typeid == TYPEID,
                    "`{}` is not an implementation type for #[extern_trait] `{}`",
                    ::core::any::type_name::<T>(),
                    stringify!(#trait_ident)
                );
            }

            /// Convert the proxy type from the implementation type.
            #[doc = #panic_doc]
            pub fn from_impl<T: #trait_ident>(value: T) -> Self {
                Self::assert_type_is_impl::<T>();
                Self(unsafe { #extern_trait::Repr::from_value(value) })
            }

            /// Convert the proxy type into the implementation type.
            #[doc = #panic_doc]
            pub fn into_impl<T: #trait_ident>(self) -> T {
                Self::assert_type_is_impl::<T>();
                unsafe {
                    #extern_trait::Repr::into_value(
                        #extern_trait::Repr::from_value(self)
                    )
                }
            }

            /// Returns a reference to the implementation type.
            #[doc = #panic_doc]
            pub fn downcast_ref<T: #trait_ident>(&self) -> &T {
                Self::assert_type_is_impl::<T>();
                unsafe { &*(self as *const Self as *const T) }
            }

            /// Returns a mutable reference to the implementation type.
            #[doc = #panic_doc]
            pub fn downcast_mut<T: #trait_ident>(&mut self) -> &mut T {
                Self::assert_type_is_impl::<T>();
                unsafe { &mut *(self as *mut Self as *mut T) }
            }
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! #macro_ident {
            ($trait:path: $ty:ty) => {
                #macro_exports

                const _: () = {
                    #[unsafe(export_name = #drop_name)]
                    unsafe fn drop(this: &mut $ty) {
                        unsafe { ::core::ptr::drop_in_place(this) };
                    }

                    #[unsafe(export_name = #typeid_name)]
                    static TYPEID: #extern_trait::__private::ConstTypeId =
                        #extern_trait::__private::ConstTypeId::of::<$ty>();
                };
            };
        }

        #[doc(hidden)]
        #[allow(unused_imports)]
        #vis use #macro_ident as #trait_ident;
    })
}

fn collect_trait_methods(input: &ItemTrait) -> Result<Vec<VerifiedSignature>> {
    input
        .items
        .iter()
        .map(|item| {
            let TraitItem::Fn(f) = item else {
                return Err(Error::new_spanned(
                    item,
                    "#[extern_trait] may only contain methods",
                ));
            };
            VerifiedSignature::try_new(&f.sig)
        })
        .collect()
}

fn generate_impl_method(
    proxy_ident: &Ident,
    export_name: &str,
    method: &VerifiedSignature,
) -> TokenStream {
    let unsafety = method.unsafety;
    let abi = &method.abi;
    let ident = &method.ident;

    let proxy: Box<Type> = parse_quote!(#proxy_ident);

    let arg_names = method.arg_names().collect::<Vec<_>>();
    let arg_types = method
        .inputs
        .iter()
        .map(|input| input.to_type(proxy.clone()))
        .collect::<Vec<_>>();
    let output = match &method.output {
        None => ReturnType::Default,
        Some(output) => ReturnType::Type(parse_quote!(->), output.to_type(proxy.clone())),
    };

    quote! {
        #unsafety #abi fn #ident(#(#arg_names: #arg_types),*) #output {
            unsafe extern "Rust" {
                #[link_name = #export_name]
                unsafe fn #ident(#(_: #arg_types),*) #output;
            }
            unsafe { #ident(#(#arg_names),*) }
        }
    }
}

fn generate_macro_export(
    extern_trait: &Path,
    trait_: Option<&Path>,
    export_name: &str,
    method: &VerifiedSignature,
) -> TokenStream {
    let unsafety = method.unsafety;
    let ident = &method.ident;

    let placeholder: Box<Type> = Box::new(Type::Verbatim(quote!($ty)));
    let repr_type: Box<Type> = parse_quote!(#extern_trait::Repr);

    // Generate arg names: _0, _1, _2, ...
    let arg_names = (0..method.inputs.len())
        .map(|i| format_ident!("_{}", i))
        .collect::<Vec<_>>();

    // For by-value Self parameters, use Repr as the extern function parameter type
    let arg_types = method
        .inputs
        .iter()
        .map(|input| {
            if input.is_self_value() {
                repr_type.clone()
            } else {
                input.to_type(placeholder.clone())
            }
        })
        .collect::<Vec<_>>();

    // For by-value Self parameters, convert from Repr to $ty
    let call_args = method
        .inputs
        .iter()
        .zip(&arg_names)
        .map(|(input, name)| {
            if input.is_self_value() {
                quote!(unsafe { #extern_trait::Repr::into_value::<$ty>(#name) })
            } else {
                quote!(#name)
            }
        })
        .collect::<Vec<_>>();

    let res = quote! { __result };

    let (ret, output) = match &method.output {
        // For by-value Self return, wrap result in Repr::from_value and use Repr as return type
        Some(output) if output.is_self_value() => (
            quote! { unsafe { #extern_trait::Repr::from_value(#res) } },
            ReturnType::Type(parse_quote!(->), repr_type.clone()),
        ),
        Some(output) => (
            res.clone(),
            ReturnType::Type(parse_quote!(->), output.to_type(placeholder.clone())),
        ),
        None => (res.clone(), ReturnType::Default),
    };

    let trait_name = trait_.map_or_else(|| quote!($trait), |t| quote!(#t));

    quote! {
        const _: () = {
            #[unsafe(export_name = #export_name)]
            fn #ident(#(#arg_names: #arg_types),*) #output {
                let #res = #unsafety {
                    <$ty as #trait_name>::#ident(#(#call_args),*)
                };
                #ret
            }
        };
    }
}

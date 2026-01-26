mod proxy;
mod sig;
mod sym;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Error, GenericArgument, Ident, ItemTrait, PathArguments, PathSegment, Result,
    TraitBoundModifier, TraitItem, Type, TypeParamBound, parse_quote,
};

use self::{proxy::Proxy, sig::VerifiedSignature, sym::Symbol};

pub fn expand(proxy: Proxy, input: ItemTrait) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            input.generics,
            "#[extern_trait] may not have generics",
        ));
    }

    let trait_name = &input.ident;
    let Some(unsafety) = &input.unsafety else {
        return Err(Error::new(
            Span::call_site(),
            "#[extern_trait] must be unsafe",
        ));
    };

    let proxy_name = &proxy.ident;
    let mut impl_content = TokenStream::new();

    let macro_name = format_ident!("__extern_trait_{}", trait_name);
    let mut macro_content = TokenStream::new();

    let sym = Symbol::new(trait_name.to_string());

    for t in &input.items {
        let TraitItem::Fn(f) = t else {
            impl_content.extend(
                Error::new_spanned(t, "#[extern_trait] may only contain methods")
                    .to_compile_error(),
            );
            continue;
        };

        let export_name = format!("{:?}", sym.clone().with_name(f.sig.ident.to_string()));

        match VerifiedSignature::try_new(&f.sig) {
            Ok(sig) => {
                impl_content.extend(generate_proxy_impl(proxy_name, &export_name, &sig));
                macro_content.extend(generate_macro_rules(None, &export_name, &sig));
            }
            Err(e) => {
                impl_content.extend(e.to_compile_error());
            }
        }
    }

    let mut extra_impls = TokenStream::new();

    for t in &input.supertraits {
        if let TypeParamBound::Trait(t) = t
            && matches!(t.modifier, TraitBoundModifier::None)
            && t.lifetimes.is_none()
            && t.path.leading_colon.is_none()
            && t.path.segments.len() == 1
        {
            let t = &t.path.segments[0];
            let PathSegment { ident, arguments } = &t;
            if ident == "Send" {
                extra_impls.extend(quote! {
                    unsafe impl Send for #proxy_name {}
                });
            } else if ident == "Sync" {
                extra_impls.extend(quote! {
                    unsafe impl Sync for #proxy_name {}
                });
            } else if ident == "AsRef"
                && let PathArguments::AngleBracketed(args) = arguments
                && let Some(GenericArgument::Type(ty)) = args.args.first()
            {
                let export_name = format!(
                    "{:?}",
                    sym.clone()
                        .with_name(format!("{}::as_ref", t.to_token_stream()))
                );
                let sig =
                    VerifiedSignature::try_new(&parse_quote!(fn as_ref(&self) -> &#ty)).unwrap();
                let impl_content = generate_proxy_impl(proxy_name, &export_name, &sig);
                extra_impls.extend(quote! {
                    impl #t for #proxy_name {
                        #impl_content
                    }
                });
                macro_content.extend(generate_macro_rules(Some(quote!(#t)), &export_name, &sig));
            }
            // TODO: support more traits
        }
    }

    let drop_name = format!("{:?}", sym.clone().with_name("drop"),);
    let reflect_name = format!("{:?}", sym.clone().with_name("reflect"),);
    let generic_doc = format!(
        "`T` must implement [`{}`] via `#[extern_trait]`.",
        trait_name
    );

    Ok(quote! {
        #input

        #proxy

        #unsafety impl #trait_name for #proxy_name {
            #impl_content
        }

        #extra_impls

        impl Drop for #proxy_name {
            fn drop(&mut self) {
                unsafe extern "Rust" {
                    #[link_name = #drop_name]
                    safe fn drop(this: *mut #proxy_name);
                }
                drop(self)
            }
        }

        impl #proxy_name {
            unsafe fn reflect<T, R>() -> extern "Rust" fn(T) -> R {
                unsafe extern "Rust" {
                    #[link_name = #reflect_name]
                    safe fn reflect(this: #proxy_name) -> #proxy_name;
                }
                unsafe {
                    ::core::mem::transmute::<_, extern "Rust" fn(T) -> R>(reflect as *const ())
                }
            }

            /// Convert the proxy type from the implementation type.
            /// # Safety
            #[doc = #generic_doc]
            pub unsafe fn from_impl<T: #trait_name>(value: T) -> Self {
                unsafe { Self::reflect::<T, #proxy_name>()(value) }
            }

            /// Convert the proxy type into the implementation type.
            /// # Safety
            #[doc = #generic_doc]
            pub unsafe fn into_impl<T: #trait_name>(self) -> T {
                unsafe { Self::reflect::<#proxy_name, T>()(self) }
            }

            /// Returns a reference to the implementation type.
            /// # Safety
            #[doc = #generic_doc]
            pub unsafe fn downcast_ref<T: #trait_name>(&self) -> &T {
                unsafe { &*(self as *const Self as *const T) }
            }

            /// Returns a mutable reference to the implementation type.
            /// # Safety
            #[doc = #generic_doc]
            pub unsafe fn downcast_mut<T: #trait_name>(&mut self) -> &mut T {
                unsafe { &mut *(self as *mut Self as *mut T) }
            }
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! #macro_name {
            ($trait:path: $ty:ty) => {
                const _: () = {
                    #macro_content

                    #[doc(hidden)]
                    #[unsafe(export_name = #drop_name)]
                    extern "Rust" fn drop(this: &mut $ty) {
                        unsafe { ::core::ptr::drop_in_place(this) };
                    }

                    #[doc(hidden)]
                    #[unsafe(export_name = #reflect_name)]
                    extern "Rust" fn reflect(this: $ty) -> $ty {
                        this
                    }
                };
            };
        }

        #[doc(hidden)]
        pub use #macro_name as #trait_name;
    })
}

fn generate_proxy_impl(
    proxy_name: &Ident,
    export_name: &str,
    sig: &VerifiedSignature,
) -> TokenStream {
    let unsafety = sig.unsafety;
    let ident = &sig.ident;

    let proxy: Box<Type> = parse_quote!(#proxy_name);

    let arg_names = sig.arg_names().collect::<Vec<_>>();
    let arg_types = sig.arg_types(proxy.clone()).collect::<Vec<_>>();
    let output = sig.return_type(proxy.clone());

    quote! {
        #unsafety fn #ident(#(#arg_names: #arg_types),*) #output {
            unsafe extern "Rust" {
                #[link_name = #export_name]
                unsafe fn #ident(#(_: #arg_types),*) #output;
            }
            unsafe {
                #ident(#(#arg_names),*)
            }
        }
    }
}

fn generate_macro_rules(
    trait_name: Option<TokenStream>,
    export_name: &str,
    sig: &VerifiedSignature,
) -> TokenStream {
    let unsafety = sig.unsafety;
    let ident = &sig.ident;

    let placeholder = Box::new(Type::Verbatim(quote!($ty)));

    let arg_names = sig.arg_names_no_self().collect::<Vec<_>>();
    let arg_types = sig.arg_types(placeholder.clone()).collect::<Vec<_>>();
    let output = sig.return_type(placeholder.clone());

    let trait_name = trait_name.unwrap_or_else(|| quote!($trait));

    quote! {
        #[doc(hidden)]
        #[unsafe(export_name = #export_name)]
        unsafe extern "Rust" fn #ident(#(#arg_names: #arg_types),*) #output {
            #unsafety {
                <$ty as #trait_name>::#ident(#(#arg_names),*)
            }
        }
    }
}

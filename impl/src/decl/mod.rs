mod proxy;
mod sig;
mod supertraits;
mod sym;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Ident, ItemTrait, Result, TraitBoundModifier, TraitItem, Type, TypeParamBound,
    parse_quote,
};

use self::{proxy::Proxy, sig::VerifiedSignature, sym::Symbol};
use crate::attr::extern_trait_path;

pub fn expand(proxy: Proxy, mut input: ItemTrait) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            input.generics,
            "#[extern_trait] may not have generics",
        ));
    }

    let extern_trait = extern_trait_path(&mut input.attrs)?;

    let unsafety = &input.unsafety;
    let trait_ident = &input.ident;
    let proxy_ident = &proxy.ident;

    let sym = Symbol::new(trait_ident.to_string());

    let mut impl_content = TokenStream::new();
    let mut macro_content = TokenStream::new();

    for t in &mut input.items {
        let TraitItem::Fn(f) = t else {
            impl_content.extend(
                Error::new_spanned(t, "#[extern_trait] may only contain methods")
                    .to_compile_error(),
            );
            continue;
        };

        let export_name = format!("{:?}", sym.clone().with_name(f.sig.ident.to_string()));

        match VerifiedSignature::try_new(Some(&mut f.attrs), &f.sig) {
            Ok(sig) => {
                impl_content.extend(generate_proxy_impl(proxy_ident, &export_name, &sig));
                macro_content.extend(generate_macro_rules(None, &export_name, &sig));
            }
            Err(e) => {
                impl_content.extend(e.to_compile_error());
            }
        }
    }

    let mut super_impls = TokenStream::new();

    for t in &input.supertraits {
        if let TypeParamBound::Trait(t) = t
            && matches!(t.modifier, TraitBoundModifier::None)
            && t.lifetimes.is_none()
            && t.path.leading_colon.is_none()
            && t.path.segments.len() == 1
        {
            let t = &t.path.segments[0];
            if let Some((impl_block, macro_rules)) =
                supertraits::generate_impl(t, proxy_ident, &sym)
            {
                super_impls.extend(impl_block);
                macro_content.extend(macro_rules);
            }
        }
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

        unsafe impl #extern_trait::ExternSafe for #proxy_ident {}

        #unsafety impl #trait_ident for #proxy_ident {
            #impl_content
        }

        #super_impls

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
            unsafe fn transmute<T, R>() -> fn(T) -> R {
                fn transmute(this: #proxy_ident) -> #proxy_ident {
                    this
                }
                unsafe {
                    ::core::mem::transmute::<_, fn(T) -> R>(transmute as *const ())
                }
            }

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
                unsafe { Self::transmute::<T, #proxy_ident>()(value) }
            }

            /// Convert the proxy type into the implementation type.
            #[doc = #panic_doc]
            pub fn into_impl<T: #trait_ident>(self) -> T {
                Self::assert_type_is_impl::<T>();
                unsafe { Self::transmute::<#proxy_ident, T>()(self) }
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
                #macro_content

                const _: () = {
                    #[unsafe(export_name = #drop_name)]
                    unsafe extern "Rust" fn drop(this: &mut $ty) {
                        unsafe { ::core::ptr::drop_in_place(this) };
                    }

                    #[unsafe(export_name = #typeid_name)]
                    static TYPEID: #extern_trait::__private::ConstTypeId =
                        #extern_trait::__private::ConstTypeId::of::<$ty>();
                };
            };
        }

        #[doc(hidden)]
        pub use #macro_ident as #trait_ident;
    })
}

fn generate_proxy_impl(
    proxy_ident: &Ident,
    export_name: &str,
    sig: &VerifiedSignature,
) -> TokenStream {
    let abi = &sig.abi;
    let unsafety = sig.unsafety;
    let ident = &sig.ident;

    let proxy: Box<Type> = parse_quote!(#proxy_ident);

    let arg_names = sig.arg_names().collect::<Vec<_>>();
    let arg_types = sig.arg_types(proxy.clone()).collect::<Vec<_>>();
    let output = sig.return_type(proxy.clone());

    quote! {
        #unsafety fn #ident(#(#arg_names: #arg_types),*) #output {
            unsafe #abi {
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
    trait_: Option<TokenStream>,
    export_name: &str,
    sig: &VerifiedSignature,
) -> TokenStream {
    let abi = &sig.abi;
    let unsafety = sig.unsafety;
    let ident = &sig.ident;

    let placeholder = Box::new(Type::Verbatim(quote!($ty)));

    let arg_names = sig.arg_names_no_self().collect::<Vec<_>>();
    let arg_types = sig.arg_types(placeholder.clone()).collect::<Vec<_>>();
    let output = sig.return_type(placeholder.clone());

    let trait_name = trait_.unwrap_or_else(|| quote!($trait));

    quote! {
        const _: () = {
            #[unsafe(export_name = #export_name)]
            #abi fn #ident(#(#arg_names: #arg_types),*) #output {
                #unsafety {
                    <$ty as #trait_name>::#ident(#(#arg_names),*)
                }
            }
        };
    }
}

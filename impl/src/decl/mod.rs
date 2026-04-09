mod supertraits;
mod symbol;
mod types;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Error, ItemTrait, Path, Result, ReturnType, TraitItem, Type, parse_quote};

use self::{
    supertraits::{SupertraitInfo, collect_supertraits},
    symbol::Symbol,
    types::VerifiedSignature,
};
use crate::args::{DeclArgs, Proxy};

struct ExpandCtx {
    // input
    extern_trait: Path,
    proxy: Proxy,
    input: ItemTrait,
    // parsed
    sym: Symbol,
    copy: bool,
    // generated
    macro_items: TokenStream,
}

impl ExpandCtx {
    fn new(args: DeclArgs, input: ItemTrait) -> Result<Self> {
        if !input.generics.params.is_empty() {
            return Err(Error::new_spanned(
                input.generics,
                "#[extern_trait] may not have generics",
            ));
        }

        let DeclArgs {
            extern_trait,
            proxy,
        } = args;
        let sym = Symbol::new(input.ident.to_string());

        Ok(Self {
            extern_trait,
            proxy,
            input,
            sym,
            copy: false,
            macro_items: TokenStream::new(),
        })
    }

    fn emit_method(&self, export_name: &str, method: &VerifiedSignature) -> TokenStream {
        let proxy_ident = &self.proxy.ident;
        let proxy: Box<Type> = parse_quote!(#proxy_ident);

        let unsafety = method.unsafety;
        let abi = &method.abi;
        let ident = &method.ident;

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

    fn emit_export(
        &self,
        trait_: Option<&Path>,
        export_name: &str,
        method: &VerifiedSignature,
    ) -> TokenStream {
        let extern_trait = &self.extern_trait;
        let repr: Box<Type> = parse_quote!(#extern_trait::Repr);

        let unsafety = method.unsafety;
        let ident = &method.ident;

        let placeholder: Box<Type> = Box::new(Type::Verbatim(quote!($ty)));

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
                    repr.clone()
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
                ReturnType::Type(parse_quote!(->), repr.clone()),
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

    fn expand_trait_impl(&mut self) -> Result<TokenStream> {
        let proxy_ident = &self.proxy.ident;
        let trait_ident = &self.input.ident;
        let unsafety = self.input.unsafety;

        let trait_methods = self
            .input
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
            .collect::<Result<Vec<_>>>()?;

        let mut impl_methods = TokenStream::new();

        for method in &trait_methods {
            let export_name = format!("{:?}", self.sym.clone().with_name(method.ident.to_string()));
            impl_methods.extend(self.emit_method(&export_name, method));
            self.macro_items
                .extend(self.emit_export(None, &export_name, method));
        }

        Ok(quote! {
            #unsafety impl #trait_ident for #proxy_ident {
                #impl_methods
            }
        })
    }

    fn expand_supertrait_impls(&mut self) -> TokenStream {
        let proxy_ident = &self.proxy.ident;

        let supertraits = collect_supertraits(&self.input.supertraits);
        let mut supertrait_impls = TokenStream::new();

        for info in &supertraits {
            let mut supertrait_impl = TokenStream::new();

            let SupertraitInfo {
                is_unsafe,
                path,
                methods,
            } = info;

            if path.is_ident("Copy") {
                self.copy = true;
            }

            for method in methods {
                let export_name = format!(
                    "{:?}",
                    self.sym.clone().with_name(format!(
                        "{}::{}",
                        path.to_token_stream(),
                        method.ident
                    ))
                );
                supertrait_impl.extend(self.emit_method(&export_name, method));
                self.macro_items
                    .extend(self.emit_export(Some(path), &export_name, method));
            }

            let unsafety = is_unsafe.then(|| quote! { unsafe });

            supertrait_impls.extend(quote! {
                #unsafety impl #path for #proxy_ident {
                    #supertrait_impl
                }
            });
        }

        supertrait_impls
    }

    fn expand_drop_impl(&mut self) -> TokenStream {
        let proxy_ident = &self.proxy.ident;
        let drop_name = format!("{:?}", self.sym.clone().with_name("drop"));

        self.macro_items.extend(quote! {
            const _: () = {
                #[unsafe(export_name = #drop_name)]
                unsafe fn drop(this: *mut $ty) {
                    unsafe { ::core::ptr::drop_in_place(this) };
                }
            };
        });

        quote! {
            impl Drop for #proxy_ident {
                fn drop(&mut self) {
                    unsafe extern "Rust" {
                        #[link_name = #drop_name]
                        unsafe fn drop(this: *mut #proxy_ident);
                    }
                    unsafe { drop(self) }
                }
            }
        }
    }

    fn expand_cast_impl(&mut self) -> TokenStream {
        let extern_trait = &self.extern_trait;
        let proxy_ident = &self.proxy.ident;
        let trait_ident = &self.input.ident;

        let typeid_name = format!("{:?}", self.sym.clone().with_name("typeid"));
        let panic_doc = format!(
            "# Panics\nPanics if the type parameter `T` is not an implementation type for \
             #[extern_trait] `{}`.",
            trait_ident
        );

        self.macro_items.extend(quote! {
            const _: () = {
                #[unsafe(export_name = #typeid_name)]
                static TYPEID: #extern_trait::__private::ConstTypeId =
                    #extern_trait::__private::ConstTypeId::of::<$ty>();
            };
        });

        quote! {
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
        }
    }

    fn expand_macro_rules(&mut self) -> TokenStream {
        let trait_ident = &self.input.ident;

        let macro_ident = format_ident!("__extern_trait_{}", trait_ident);
        let macro_content = &self.macro_items;
        let vis = &self.input.vis;

        quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #macro_ident {
                ($trait:path: $ty:ty) => {
                    #macro_content
                };
            }

            #[doc(hidden)]
            #[allow(unused_imports)]
            #vis use #macro_ident as #trait_ident;
        }
    }

    fn expand(&mut self) -> Result<TokenStream> {
        let trait_impl = self.expand_trait_impl()?;
        let supertrait_impls = self.expand_supertrait_impls();
        let drop_impl = (!self.copy).then(|| self.expand_drop_impl());
        let cast_impl = self.expand_cast_impl();
        let macro_rules = self.expand_macro_rules();

        let input = &self.input;
        let proxy = self.proxy.expand(&self.extern_trait);

        Ok(quote! {
            #input

            #proxy

            #trait_impl

            #supertrait_impls

            #drop_impl

            #cast_impl

            #macro_rules
        })
    }
}

pub fn expand(args: DeclArgs, input: ItemTrait) -> Result<TokenStream> {
    ExpandCtx::new(args, input)?.expand()
}

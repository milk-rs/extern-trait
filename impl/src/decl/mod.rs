mod supertraits;
mod symbol;
mod types;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Error, Ident, ItemTrait, Path, Result, ReturnType, TraitItem, Type, parse_quote,
    spanned::Spanned,
};

use self::{
    supertraits::{SupertraitInfo, collect_supertraits},
    symbol::Symbol,
    types::VerifiedSignature,
};
use crate::{
    args::{DeclArgs, Proxy, ReprType},
    decl::types::{MaybeSelf, arg_names, make_return_type},
};
// ---------------------------------------------------------------------------
// MethodInfo: unified representation for trait + supertrait methods
// ---------------------------------------------------------------------------

struct MethodInfo {
    sig: VerifiedSignature,
    /// `None` for trait's own methods, `Some(path)` for supertrait methods.
    supertrait_path: Option<Path>,
}

impl MethodInfo {
    /// VTable field name: `method` for own methods, `__Trait_method` for supertrait.
    fn field_name(&self) -> Ident {
        match &self.supertrait_path {
            None => self.sig.ident.clone(),
            Some(path) => {
                let last = path.segments.last().unwrap();
                format_ident!("__{}_{}", last.ident, self.sig.ident)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ExpandCtx
// ---------------------------------------------------------------------------

struct ExpandCtx {
    // input
    extern_trait: Path,
    repr_type: ReprType,
    proxy: Proxy,
    input: ItemTrait,
    // parsed
    sym: Symbol,
    copy: bool,
    supertraits: Vec<SupertraitInfo>,
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
            repr_type,
        } = args;
        let sym = Symbol::new(input.ident.to_string());

        Ok(Self {
            extern_trait,
            proxy,
            input,
            sym,
            repr_type,
            copy: false,
            supertraits: Vec::new(),
        })
    }

    // -----------------------------------------------------------------------
    // Collect all methods
    // -----------------------------------------------------------------------

    fn collect_methods(&mut self) -> Result<Vec<MethodInfo>> {
        let mut methods = Vec::new();

        // Trait's own methods
        for item in &self.input.items {
            let TraitItem::Fn(f) = item else {
                return Err(Error::new_spanned(
                    item,
                    "#[extern_trait] may only contain methods",
                ));
            };
            methods.push(MethodInfo {
                sig: VerifiedSignature::try_new(&f.sig)?,
                supertrait_path: None,
            });
        }

        // Supertrait methods
        self.supertraits = collect_supertraits(&self.input.supertraits);
        for info in &self.supertraits {
            if info.path.is_ident("Copy") {
                self.copy = true;
            }
            for sig in &info.methods {
                methods.push(MethodInfo {
                    sig: sig.clone(),
                    supertrait_path: Some(info.path.clone()),
                });
            }
        }

        Ok(methods)
    }

    // -----------------------------------------------------------------------
    // VTable struct generation
    // -----------------------------------------------------------------------

    fn vtable_ident(&self) -> Ident {
        format_ident!("__{}VTable", self.input.ident)
    }

    fn vtable_symbol(&self) -> String {
        format!("{:#?}", self.sym)
    }

    /// `extern_trait::Repr` as a syn `Type`.
    fn repr_type(&self) -> Type {
        self.repr_type.to_syn_type()
    }

    /// Build a `ReturnType`, replacing by-value `Self` with `Repr`.
    fn return_type(&self, output: &Option<MaybeSelf>, self_type: &Type) -> ReturnType {
        if output.as_ref().is_some_and(|o| o.is_self_value()) {
            let repr = self.repr_type();
            make_return_type(output, &repr)
        } else {
            make_return_type(output, self_type)
        }
    }

    /// Build a fn pointer type for a VTable method field.
    ///
    /// `self_type` is substituted for ref/ptr Self. By-value Self uses `Repr`.
    fn method_fn_type(&self, sig: &VerifiedSignature, self_type: &Type) -> TokenStream {
        let VerifiedSignature {
            unsafety,
            ident: _,
            inputs,
            output,
        } = sig;

        let repr = self.repr_type();

        let arg_types: Vec<_> = inputs
            .iter()
            .map(|input| {
                if input.is_self_value() {
                    Box::new(repr.clone())
                } else {
                    input.to_type(self_type)
                }
            })
            .collect();

        let output = self.return_type(output, self_type);

        quote! { #unsafety fn(#(#arg_types),*) #output }
    }

    /// Emit a `#[repr(C)]` VTable struct definition.
    ///
    /// `self_type` is the type substituted for ref/ptr Self and drop pointer.
    /// By-value Self always uses `Repr`.
    fn emit_vtable_struct(&self, methods: &[MethodInfo], self_type: &Type) -> TokenStream {
        let extern_trait = &self.extern_trait;
        let vtable_ident = self.vtable_ident();

        let method_fields: Vec<_> = methods
            .iter()
            .map(|m| {
                let field_name = m.field_name();
                let fn_type = self.method_fn_type(&m.sig, self_type);
                quote! { #field_name: #fn_type }
            })
            .collect();

        quote! {
            #[repr(C)]
            #[allow(non_snake_case)]
            struct #vtable_ident {
                typeid: #extern_trait::__private::ConstTypeId,
                drop: unsafe fn(*mut #self_type),
                #(#method_fields),*
            }
        }
    }

    // -----------------------------------------------------------------------
    // Proxy-side: extern static + trait/supertrait impls
    // -----------------------------------------------------------------------

    fn emit_extern_vtable(&self) -> TokenStream {
        let vtable_ident = self.vtable_ident();
        let vtable_symbol = self.vtable_symbol();

        quote! {
            unsafe extern "Rust" {
                #[link_name = #vtable_symbol]
                safe static VT: #vtable_ident;
            }
        }
    }

    fn emit_trait_impl(&self, methods: &[MethodInfo]) -> TokenStream {
        let proxy_ident = &self.proxy.ident;
        let trait_ident = &self.input.ident;
        let unsafety = self.input.unsafety;

        let impl_methods: Vec<_> = methods
            .iter()
            .filter(|m| m.supertrait_path.is_none())
            .map(|m| self.emit_method_body(m))
            .collect();

        quote! {
            #unsafety impl #trait_ident for #proxy_ident {
                #(#impl_methods)*
            }
        }
    }

    fn emit_supertrait_impls(&self, methods: &[MethodInfo]) -> TokenStream {
        let proxy_ident = &self.proxy.ident;
        let mut impls = TokenStream::new();

        for info in &self.supertraits {
            let SupertraitInfo {
                is_unsafe,
                path,
                methods: _,
            } = info;

            let supertrait_methods: Vec<_> = methods
                .iter()
                .filter(|m| m.supertrait_path.as_ref().is_some_and(|p| p == path))
                .map(|m| self.emit_method_body(m))
                .collect();

            let unsafety = is_unsafe.then(|| quote! { unsafe });

            impls.extend(quote! {
                #unsafety impl #path for #proxy_ident {
                    #(#supertrait_methods)*
                }
            });
        }

        impls
    }

    /// Generate a single method body that calls through the VTable.
    fn emit_method_body(&self, method: &MethodInfo) -> TokenStream {
        let _ = &self.extern_trait;
        let proxy_ident = &self.proxy.ident;
        let proxy_type: Type = parse_quote!(#proxy_ident);
        let repr_type = self.repr_type();

        let VerifiedSignature {
            unsafety,
            ident,
            inputs,
            output,
        } = &method.sig;

        let arg_names: Vec<_> = arg_names(inputs);
        let arg_types: Vec<_> = inputs
            .iter()
            .map(|input| input.to_type(&proxy_type))
            .collect();

        // Convert by-value Self args: ProxyType → Repr (transparent transmute)
        let call_args: Vec<_> = inputs
            .iter()
            .zip(&arg_names)
            .map(|(input, name)| {
                if input.is_self_value() {
                    quote!(unsafe { #repr_type::from_value(#name) })
                } else {
                    quote!(#name)
                }
            })
            .collect();

        let field_name = method.field_name();
        let body = quote! { (VT.#field_name)(#(#call_args),*) };

        // Wrap Repr result back to ProxyType if by-value Self return
        let body = if output.as_ref().is_some_and(|o| o.is_self_value()) {
            quote! { #proxy_ident(#body) }
        } else {
            body
        };

        let output = make_return_type(output, &proxy_type);

        quote! {
            #unsafety fn #ident(#(#arg_names: #arg_types),*) #output {
                #body
            }
        }
    }

    // -----------------------------------------------------------------------
    // Drop impl
    // -----------------------------------------------------------------------

    fn emit_drop_impl(&self) -> TokenStream {
        let proxy_ident = &self.proxy.ident;

        quote! {
            impl Drop for #proxy_ident {
                fn drop(&mut self) {
                    unsafe { (VT.drop)(self) }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Cast methods (from_impl, into_impl, downcast_ref, downcast_mut)
    // -----------------------------------------------------------------------

    fn emit_cast_impl(&self) -> TokenStream {
        let extern_trait = &self.extern_trait;
        let proxy_ident = &self.proxy.ident;
        let trait_ident = &self.input.ident;
        let repr_type = &self.repr_type;

        let panic_doc = format!(
            "# Panics\nPanics if the type parameter `T` is not an implementation type for \
             #[extern_trait] `{}`.",
            trait_ident
        );

        quote! {
            impl #proxy_ident {
                fn assert_type_is_impl<T: #trait_ident>() {
                    let typeid = #extern_trait::__private::ConstTypeId::of::<T>();
                    assert!(
                        typeid == VT.typeid,
                        "`{}` is not an implementation type for #[extern_trait] `{}`",
                        ::core::any::type_name::<T>(),
                        stringify!(#trait_ident)
                    );
                }

                /// Convert the proxy type from the implementation type.
                #[doc = #panic_doc]
                pub fn from_impl<T: #trait_ident>(value: T) -> Self {
                    Self::assert_type_is_impl::<T>();
                    Self(unsafe { #repr_type::from_value(value) })
                }

                /// Convert the proxy type into the implementation type.
                #[doc = #panic_doc]
                pub fn into_impl<T: #trait_ident>(self) -> T {
                    Self::assert_type_is_impl::<T>();
                    unsafe {
                        #repr_type::into_value(
                            #repr_type::from_value(self)
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

    // -----------------------------------------------------------------------
    // Impl-side: macro_rules with VTable struct + static init
    // -----------------------------------------------------------------------

    fn emit_macro_rules(&self, methods: &[MethodInfo]) -> TokenStream {
        let trait_ident = &self.input.ident;
        let macro_ident = format_ident!("__extern_trait_{}", trait_ident);
        let vis = &self.input.vis;

        let vtable_ident = self.vtable_ident();
        let vtable_symbol = self.vtable_symbol();

        let placeholder: Type = Type::Verbatim(quote!($ty));
        let vtable_struct = self.emit_vtable_struct(methods, &placeholder);
        let vtable_init = self.emit_vtable_init(methods, &placeholder);

        // size assertion, quoted to point to `ptr_count` expression
        let size_assertion: TokenStream = {
            let expected_ptr_count = &self.repr_type.ptr_count();
            let assert_msg1 = format!(
                "#[extern_trait] {trait_ident} declared with ptr_count={expected_ptr_count}, but \
                 impl for ",
            );
            quote_spanned! {expected_ptr_count.span() =>
                assert!(
                    $actual_ptr_count == #expected_ptr_count,
                    concat!(
                        #assert_msg1,
                        stringify!($ty),
                        " has ptr_count=",
                        stringify!($actual_ptr_count)
                    )
                );
            }
        };
        quote! {
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #macro_ident {
                ($trait:path: $ty:ty, ptr_count = $actual_ptr_count:expr) => {
                    const _: () = {
                        #vtable_struct

                        #size_assertion

                        #[unsafe(export_name = #vtable_symbol)]
                        static VT: #vtable_ident = #vtable_init;
                    };
                };
            }

            #[doc(hidden)]
            #[allow(unused_imports)]
            #vis use #macro_ident as #trait_ident;
        }
    }

    /// Generate the VTable static initializer expression.
    fn emit_vtable_init(&self, methods: &[MethodInfo], self_type: &Type) -> TokenStream {
        let extern_trait = &self.extern_trait;
        let vtable_ident = self.vtable_ident();

        let method_inits: Vec<_> = methods
            .iter()
            .map(|m| {
                let field_name = m.field_name();
                let init = self.emit_vtable_field_init(m, self_type);
                quote! { #field_name: #init }
            })
            .collect();

        quote! {
            #vtable_ident {
                typeid: #extern_trait::__private::ConstTypeId::of::<$ty>(),
                drop: |this: *mut $ty| unsafe { ::core::ptr::drop_in_place(this) },
                #(#method_inits),*
            }
        }
    }

    /// Generate a single VTable field initializer closure for the impl side.
    fn emit_vtable_field_init(&self, method: &MethodInfo, self_type: &Type) -> TokenStream {
        let _ = self.extern_trait;
        let repr_type = self.repr_type();

        let MethodInfo {
            sig,
            supertrait_path,
        } = method;
        let VerifiedSignature {
            unsafety,
            ident,
            inputs,
            output,
        } = sig;

        let repr = self.repr_type();

        // Parameter names: _0, _1, _2, ...
        let arg_names: Vec<_> = (0..inputs.len()).map(|i| format_ident!("_{}", i)).collect();

        // Parameter types (same mapping as VTable struct fields)
        let arg_types: Vec<_> = inputs
            .iter()
            .map(|input| {
                if input.is_self_value() {
                    Box::new(repr.clone())
                } else {
                    input.to_type(self_type)
                }
            })
            .collect();

        // Convert arguments: by-value Self → Repr::into_value, otherwise pass through
        let call_args: Vec<_> = inputs
            .iter()
            .zip(&arg_names)
            .map(|(input, name)| {
                if input.is_self_value() {
                    quote!(unsafe { #repr_type::into_value::<$ty>(#name) })
                } else {
                    quote!(#name)
                }
            })
            .collect();

        // Trait path for qualified call
        let trait_name = match &supertrait_path {
            None => quote!($trait),
            Some(path) => quote!(#path),
        };

        let body = quote! {
            #unsafety { <$ty as #trait_name>::#ident(#(#call_args),*) }
        };

        let body = if output.as_ref().is_some_and(|o| o.is_self_value()) {
            quote! {
                let __result = #body;
                unsafe { #repr_type::from_value(__result) }
            }
        } else {
            body
        };

        quote! {
            |#(#arg_names: #arg_types),*| {
                #body
            }
        }
    }

    // -----------------------------------------------------------------------
    // Top-level expand
    // -----------------------------------------------------------------------

    fn expand(&mut self) -> Result<TokenStream> {
        let methods = self.collect_methods()?;

        let input = &self.input;
        let proxy = self.proxy.expand(&self.repr_type);

        // Proxy-side vtable struct
        let proxy_ident = &self.proxy.ident;
        let proxy_type: Type = parse_quote!(#proxy_ident);
        let vtable_struct = self.emit_vtable_struct(&methods, &proxy_type);

        // Extern vtable declaration
        let extern_vtable = self.emit_extern_vtable();

        // Trait impl
        let trait_impl = self.emit_trait_impl(&methods);

        // Supertrait impls
        let supertrait_impls = self.emit_supertrait_impls(&methods);

        // Drop impl (skip for Copy types)
        let drop_impl = (!self.copy).then(|| self.emit_drop_impl());

        // Cast methods
        let cast_impl = self.emit_cast_impl();

        // macro_rules
        let macro_rules = self.emit_macro_rules(&methods);

        Ok(quote! {
            #input

            #proxy

            const _: () = {
                #vtable_struct

                #extern_vtable

                #trait_impl

                #supertrait_impls

                #drop_impl

                #cast_impl
            };

            #macro_rules
        })
    }
}

pub fn expand(args: DeclArgs, input: ItemTrait) -> Result<TokenStream> {
    ExpandCtx::new(args, input)?.expand()
}

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Abi, Error, FnArg, GenericArgument, Ident, Lifetime, PathArguments, Result, ReturnType,
    Signature, Token, Type, TypePtr, TypeReference, parse_quote,
};

#[derive(Debug, Clone)]
pub enum SelfKind {
    Value,
    Ptr {
        star_token: Token![*],
        const_token: Option<Token![const]>,
        mutability: Option<Token![mut]>,
    },
    Ref {
        and_token: Token![&],
        lifetime: Option<Lifetime>,
        mutability: Option<Token![mut]>,
    },
}

impl SelfKind {
    fn to_type(&self, elem: Box<Type>) -> Box<Type> {
        match self {
            SelfKind::Value => elem,
            SelfKind::Ptr {
                star_token,
                const_token,
                mutability,
            } => Box::new(Type::Ptr(TypePtr {
                star_token: *star_token,
                const_token: *const_token,
                mutability: *mutability,
                elem,
            })),
            SelfKind::Ref {
                and_token,
                lifetime,
                mutability,
            } => Box::new(Type::Reference(TypeReference {
                and_token: *and_token,
                lifetime: lifetime.clone(),
                mutability: *mutability,
                elem,
            })),
        }
    }
}

pub trait TypeExt {
    fn contains_self(&self) -> bool;
    fn self_kind(&self) -> Option<SelfKind>;
}

impl TypeExt for Type {
    fn contains_self(&self) -> bool {
        match self {
            Type::Array(arr) => arr.elem.contains_self(),
            Type::BareFn(f) => {
                for arg in &f.inputs {
                    if arg.ty.contains_self() {
                        return true;
                    }
                }
                if let ReturnType::Type(_, ret) = &f.output
                    && ret.contains_self()
                {
                    return true;
                }
                false
            }
            Type::Group(group) => group.elem.contains_self(),
            Type::Paren(paren) => paren.elem.contains_self(),
            Type::Path(path) => {
                if let Some(qself) = &path.qself
                    && qself.ty.contains_self()
                {
                    return true;
                }
                for segment in &path.path.segments {
                    if segment.ident == "Self" {
                        return true;
                    }
                    match &segment.arguments {
                        PathArguments::None => {}
                        PathArguments::AngleBracketed(args) => {
                            for arg in &args.args {
                                if let GenericArgument::Type(ty) = arg
                                    && ty.contains_self()
                                {
                                    return true;
                                }
                            }
                        }
                        PathArguments::Parenthesized(args) => {
                            for arg in &args.inputs {
                                if arg.contains_self() {
                                    return true;
                                }
                            }
                            if let ReturnType::Type(_, ret) = &args.output
                                && ret.contains_self()
                            {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            Type::Ptr(ptr) => ptr.elem.contains_self(),
            Type::Reference(r) => r.elem.contains_self(),
            Type::Slice(slice) => slice.elem.contains_self(),
            Type::Tuple(tpl) => {
                for elem in &tpl.elems {
                    if elem.contains_self() {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn self_kind(&self) -> Option<SelfKind> {
        let self_ty = parse_quote!(Self);

        if *self == self_ty {
            Some(SelfKind::Value)
        } else if let Type::Ptr(TypePtr {
            star_token,
            const_token,
            mutability,
            elem,
        }) = self
        {
            if **elem == self_ty {
                Some(SelfKind::Ptr {
                    star_token: *star_token,
                    const_token: *const_token,
                    mutability: *mutability,
                })
            } else {
                None
            }
        } else if let Type::Reference(TypeReference {
            and_token,
            lifetime,
            mutability,
            elem,
        }) = self
        {
            if **elem == self_ty {
                Some(SelfKind::Ref {
                    and_token: *and_token,
                    lifetime: lifetime.clone(),
                    mutability: *mutability,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum MaybeSelf {
    Self_(SelfKind),
    Typed(Box<Type>),
}

impl MaybeSelf {
    pub fn to_type(&self, elem: Box<Type>) -> Box<Type> {
        match self {
            MaybeSelf::Self_(kind) => kind.to_type(elem),
            MaybeSelf::Typed(ty) => ty.clone(),
        }
    }

    pub fn is_self_value(&self) -> bool {
        matches!(self, MaybeSelf::Self_(SelfKind::Value))
    }
}

#[derive(Debug, Clone)]
pub struct VerifiedSignature {
    pub unsafety: Option<Token![unsafe]>,
    pub abi: Option<Abi>,
    pub ident: Ident,
    pub inputs: Vec<MaybeSelf>,
    pub output: Option<MaybeSelf>,
}

impl VerifiedSignature {
    pub fn try_new(sig: &Signature) -> Result<Self> {
        if sig.constness.is_some() {
            return Err(Error::new_spanned(
                sig.constness,
                "#[extern_trait] does not support const functions",
            ));
        }
        if sig.asyncness.is_some() {
            return Err(Error::new_spanned(
                sig.asyncness,
                "#[extern_trait] does not support async functions",
            ));
        }
        if !sig.generics.params.is_empty() {
            return Err(Error::new_spanned(
                &sig.generics,
                "#[extern_trait] does not support generic functions",
            ));
        }
        if sig.generics.where_clause.is_some() {
            return Err(Error::new_spanned(
                &sig.generics.where_clause,
                "#[extern_trait] does not support where clauses",
            ));
        }
        if sig.variadic.is_some() {
            return Err(Error::new_spanned(
                &sig.variadic,
                "#[extern_trait] does not support variadic functions",
            ));
        }

        let inputs = sig
            .inputs
            .iter()
            .map(|arg| match arg {
                FnArg::Receiver(arg) => arg.ty.clone(),
                FnArg::Typed(arg) => arg.ty.clone(),
            })
            .map(|ty| {
                if ty.contains_self() {
                    if let Some(kind) = ty.self_kind() {
                        Ok(MaybeSelf::Self_(kind))
                    } else {
                        Err(Error::new_spanned(
                            ty,
                            "#[extern_trait] too complex `Self` type",
                        ))
                    }
                } else {
                    Ok(MaybeSelf::Typed(ty.clone()))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let output = match &sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(if ty.contains_self() {
                let Some(kind) = ty.self_kind() else {
                    return Err(Error::new_spanned(
                        ty,
                        "#[extern_trait] too complex `Self` type",
                    ));
                };
                MaybeSelf::Self_(kind)
            } else {
                MaybeSelf::Typed(ty.clone())
            }),
        };

        Ok(Self {
            unsafety: sig.unsafety,
            abi: sig.abi.clone(),
            ident: sig.ident.clone(),
            inputs,
            output,
        })
    }

    pub fn arg_names(&self) -> impl Iterator<Item = Ident> {
        self.inputs.iter().enumerate().map(|(i, arg)| match arg {
            // Only the first parameter can use `self` keyword
            MaybeSelf::Self_(_) if i == 0 => format_ident!("self"),
            _ => format_ident!("_{}", i),
        })
    }
}

impl ToTokens for VerifiedSignature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let unsafety = &self.unsafety;
        let abi = &self.abi;
        let ident = &self.ident;
        let self_type: Box<Type> = parse_quote!(Self);
        let arg_names = self.arg_names().collect::<Vec<_>>();
        let arg_types = self
            .inputs
            .iter()
            .map(|input| input.to_type(self_type.clone()))
            .collect::<Vec<_>>();
        let output: ReturnType = match &self.output {
            None => ReturnType::Default,
            Some(output) => ReturnType::Type(parse_quote!(->), output.to_type(self_type.clone())),
        };

        tokens.extend(quote! {
            #unsafety #abi fn #ident(#(#arg_names: #arg_types),*) #output
        });
    }
}

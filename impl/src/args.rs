use std::fmt::{Display, Formatter};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Ident, Path, Token, Type, Visibility,
    parse::{Parse, ParseStream, Result},
    parse_quote,
};

use crate::utils::ParsedLitInt;

/// The integer `ptr_count`, resulting from calling [ReprType::ptr_count].
///
/// This is a different type from `TokenStream` mainly so that our `Display` impl
/// can avoid the spaces that `TokenStream` display adds.
pub struct ExpandedPtrCount<'a>(&'a ReprType);
impl ToTokens for ExpandedPtrCount<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }
    fn to_token_stream(&self) -> TokenStream {
        let extern_trait = &self.0.extern_trait;
        match self.0.ptr_count {
            Some(ref count) => quote!(#count),
            None => quote!(#extern_trait::DEFAULT_PTR_COUNT),
        }
    }
}
/// Displays the ptr count in a user-friendly way,
/// without the unnecessary spaces that [`TokenStream`] would add.
///
/// Unlike the [`ToTokens`] impl,
/// this ignores custom crate paths and always uses "extern_trait::"
/// This is fine since this is just for display purposes.
/// Supporting arbitrary paths would require us to support printing types.
impl Display for ExpandedPtrCount<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.ptr_count {
            Some(ref value) => write!(f, "{}", value.value),
            None => f.write_str("extern_trait::DEFAULT_PTR_COUNT"),
        }
    }
}

#[derive(Clone)]
pub struct ReprType {
    pub extern_trait: Path,
    pub ptr_count: Option<ParsedLitInt<usize>>,
}
impl ReprType {
    pub fn ptr_count(&self) -> ExpandedPtrCount<'_> {
        ExpandedPtrCount(self)
    }
    pub fn to_syn_type(&self) -> Type {
        parse_quote!(#self)
    }
}
impl Default for ReprType {
    fn default() -> Self {
        ReprType {
            extern_trait: parse_quote!(::extern_trait),
            ptr_count: None,
        }
    }
}
impl ToTokens for ReprType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }
    fn to_token_stream(&self) -> TokenStream {
        let extern_trait = &self.extern_trait;
        let ptr_count = self.ptr_count();
        quote!(#extern_trait::ReprN::<{ #ptr_count }>)
    }
}

/// Arguments for `#[extern_trait(...)]` on a trait declaration.
///
/// Supports the following forms:
/// - `#[extern_trait(ProxyName)]`
/// - `#[extern_trait(pub ProxyName)]`
/// - `#[extern_trait(ptr_count = N, pub ProxyName)]`
/// - `#[extern_trait(crate = path, ProxyName)]`
/// - `#[extern_trait(crate = path, pub ProxyName)]`
/// - `#[extern_trait(crate = path, ptr_count = N, pub ProxyName)]`
pub struct DeclArgs {
    pub extern_trait: Path,
    pub proxy: Proxy,
    pub repr_type: ReprType,
}

/// Arguments for `#[extern_trait(...)]` on an impl block.
///
/// Supports the following forms:
/// - `#[extern_trait]`
/// - `#[extern_trait(crate = path)]`
/// - `#[extern_trait(ptr_count = N)]`
/// - `#[extern_trait(ptr_count = N, crate = path)]`
pub struct ImplArgs {
    pub extern_trait: Path,
    pub repr_type: ReprType,
}

pub struct Proxy {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
}

impl Parse for DeclArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let kwargs = parse_kwargs(input)?;

        let proxy = Proxy {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            ident: input.parse()?,
        };

        Ok(DeclArgs {
            extern_trait: kwargs.extern_trait(),
            repr_type: kwargs.repr_type(),
            proxy,
        })
    }
}

impl Parse for ImplArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let kwargs = parse_kwargs(input)?;

        Ok(ImplArgs {
            extern_trait: kwargs.extern_trait(),
            repr_type: kwargs.repr_type(),
        })
    }
}

impl Proxy {
    pub fn expand(&self, repr_type: &ReprType) -> TokenStream {
        let Proxy { attrs, vis, ident } = self;

        quote::quote! {
            #(#attrs)*
            #[repr(transparent)]
            #vis struct #ident(#repr_type);
        }
    }
}

mod kw {
    syn::custom_keyword!(ptr_count);
}
/// The result of [parse_kwargs], shared between [DeclArgs] and [ImplArgs]
#[derive(Default, Clone)]
struct SharedKeywordArgs {
    pub crate_path: Option<Path>,
    pub ptr_count: Option<ParsedLitInt<usize>>,
}
impl SharedKeywordArgs {
    fn is_empty(&self) -> bool {
        matches!(
            self,
            SharedKeywordArgs {
                crate_path: None,
                ptr_count: None
            }
        )
    }
    fn repr_type(&self) -> ReprType {
        ReprType {
            extern_trait: self.extern_trait(),
            ptr_count: self.ptr_count.clone(),
        }
    }
    fn extern_trait(&self) -> Path {
        self.crate_path
            .clone()
            .unwrap_or_else(|| parse_quote!(::extern_trait))
    }
    fn parse_single(&self, input: ParseStream) -> Result<Option<Self>> {
        macro_rules! do_parse {
            // the `$kw` must be wrapped in parens so that it is parsed as a tt
            // it must be parsed as a tt since it is used as both expr and ty
            ($field:ident, $desc:literal, ($($kw:tt)*), $parse:block) => {
                // need to
                if input.peek($($kw)*) && input.peek2(Token![=]) {
                    let kw = input.parse::<$($kw)*>()?;
                    input.parse::<Token![=]>()?;
                    let res = $parse;
                    return if self.$field.is_some() {
                        Err(syn::Error::new_spanned(
                            kw,
                            format_args!("Conflicting {} arguments", $desc),
                        ))
                    } else {
                        Ok(Some(Self {
                            $field: Some(res),
                            ..self.clone()
                        }))
                    };
                }
            };
        }
        do_parse!(crate_path, "crate path", (Token![crate]), {
            input.call(Path::parse_mod_style)?
        });
        do_parse!(ptr_count, "ptr count (size)", (kw::ptr_count), {
            input.parse::<ParsedLitInt<usize>>()?
        });
        Ok(None)
    }
}

/// Parse optional `crate = path` and `ptr_count = N` from the input stream,
/// separated by commas.
///
/// Will consume a trailing comma (if any).
/// Parsing stops at the first unrecognized keyword argument.
fn parse_kwargs(input: ParseStream) -> Result<SharedKeywordArgs> {
    let mut kwargs = SharedKeywordArgs::default();
    loop {
        if !kwargs.is_empty() {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                // no comma => nothing more to parse
                return Ok(kwargs);
            }
        }
        match kwargs.parse_single(input)? {
            Some(updated) => {
                kwargs = updated;
            }
            None => break Ok(kwargs),
        }
    }
}

use proc_macro2::{Span, TokenStream};
use syn::{
    Attribute, Error, Ident, Path, Token, Visibility,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
};

/// Parsed arguments for `#[extern_trait(...)]`.
///
/// Supports the following forms:
/// - `#[extern_trait]`
/// - `#[extern_trait(ProxyName)]`
/// - `#[extern_trait(pub ProxyName)]`
/// - `#[extern_trait(crate = path)]`
/// - `#[extern_trait(crate = path, ProxyName)]`
/// - `#[extern_trait(ProxyName, crate = path)]`
pub struct Args {
    extern_trait: Path,
    proxy: Option<Proxy>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<Arg, Token![,]>::parse_terminated(input)?;
        let mut extern_trait = None;
        let mut proxy = None;

        for arg in args {
            match arg {
                Arg::Crate(path) => {
                    if extern_trait.is_some() {
                        return Err(Error::new_spanned(path, "duplicate `crate` argument"));
                    }
                    extern_trait = Some(path);
                }
                Arg::Proxy(value) => {
                    if proxy.is_some() {
                        return Err(Error::new_spanned(
                            value.ident,
                            "proxy type specified more than once",
                        ));
                    }
                    proxy = Some(value);
                }
            }
        }

        Ok(Args {
            extern_trait: extern_trait.unwrap_or_else(|| parse_quote!(::extern_trait)),
            proxy,
        })
    }
}

/// Validated arguments for `#[extern_trait(...)]` on a trait declaration.
pub struct TraitArgs {
    pub extern_trait: Path,
    pub proxy: Proxy,
}

impl TryFrom<Args> for TraitArgs {
    type Error = Error;

    fn try_from(args: Args) -> Result<Self> {
        let proxy = args.proxy.ok_or_else(|| {
            Error::new(
                Span::call_site(),
                "#[extern_trait] on a trait requires a proxy type",
            )
        })?;

        Ok(TraitArgs {
            extern_trait: args.extern_trait,
            proxy,
        })
    }
}

/// Validated arguments for `#[extern_trait(...)]` on an impl block.
pub struct ImplArgs {
    pub extern_trait: Path,
}

impl TryFrom<Args> for ImplArgs {
    type Error = Error;

    fn try_from(args: Args) -> Result<Self> {
        if let Some(proxy) = args.proxy {
            return Err(Error::new_spanned(
                proxy.ident,
                "proxy type is only supported on trait declarations",
            ));
        }

        Ok(ImplArgs {
            extern_trait: args.extern_trait,
        })
    }
}

pub struct Proxy {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
}

impl Proxy {
    pub fn expand(&self, extern_trait: &Path) -> TokenStream {
        let Proxy { attrs, vis, ident } = self;

        quote::quote! {
            #(#attrs)*
            #[repr(transparent)]
            #vis struct #ident(#extern_trait::Repr);
        }
    }
}

enum Arg {
    Crate(Path),
    Proxy(Proxy),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![crate]) {
            input.parse::<Token![crate]>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Crate(input.call(Path::parse_mod_style)?));
        }

        if input.peek(Ident) && input.peek2(Token![=]) {
            let ident: Ident = input.parse()?;
            return Err(Error::new_spanned(
                ident,
                "unknown #[extern_trait] argument",
            ));
        }

        Ok(Self::Proxy(Proxy {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            ident: input.parse()?,
        }))
    }
}

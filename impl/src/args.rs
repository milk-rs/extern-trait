use proc_macro2::TokenStream;
use syn::{
    Attribute, Ident, Path, Token, Visibility,
    parse::{Parse, ParseStream, Result},
    parse_quote,
};

/// Arguments for `#[extern_trait(...)]` on a trait declaration.
///
/// Supports the following forms:
/// - `#[extern_trait(ProxyName)]`
/// - `#[extern_trait(pub ProxyName)]`
/// - `#[extern_trait(crate = path, ProxyName)]`
/// - `#[extern_trait(crate = path, pub ProxyName)]`
pub struct DeclArgs {
    pub extern_trait: Path,
    pub proxy: Proxy,
}

/// Arguments for `#[extern_trait(...)]` on an impl block.
///
/// Supports the following forms:
/// - `#[extern_trait]`
/// - `#[extern_trait(crate = path)]`
pub struct ImplArgs {
    pub extern_trait: Path,
}

pub struct Proxy {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
}

impl Parse for DeclArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let extern_trait = parse_crate_path(input)?;

        // If we consumed `crate = path`, expect a comma before the proxy
        if extern_trait.is_some() {
            input.parse::<Token![,]>()?;
        }

        let proxy = Proxy {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            ident: input.parse()?,
        };

        Ok(DeclArgs {
            extern_trait: extern_trait.unwrap_or_else(|| parse_quote!(::extern_trait)),
            proxy,
        })
    }
}

impl Parse for ImplArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let extern_trait = parse_crate_path(input)?;

        Ok(ImplArgs {
            extern_trait: extern_trait.unwrap_or_else(|| parse_quote!(::extern_trait)),
        })
    }
}

impl Proxy {
    pub fn expand(&self, extern_trait: &Path) -> TokenStream {
        let Proxy { attrs, vis, ident } = self;

        quote::quote! {
            #(#attrs)*
            #[repr(transparent)]
            #vis struct #ident(#extern_trait::Repr);

            unsafe impl #extern_trait::IntRegRepr for #ident {}
        }
    }
}

/// Parse optional `crate = path` from the input stream.
fn parse_crate_path(input: ParseStream) -> Result<Option<Path>> {
    if input.peek(Token![crate]) {
        input.parse::<Token![crate]>()?;
        input.parse::<Token![=]>()?;
        let path = input.call(Path::parse_mod_style)?;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

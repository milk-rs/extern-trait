use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    Attribute, Ident, Result, Visibility,
    parse::{Parse, ParseStream},
};

pub struct Proxy {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
}

impl Parse for Proxy {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;

        Ok(Proxy { attrs, vis, ident })
    }
}

impl ToTokens for Proxy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Proxy { attrs, vis, ident } = self;

        tokens.append_all(quote! {
            #(#attrs)*
            #[repr(C)]
            #vis struct #ident(*const (), *const ());
        });
    }
}

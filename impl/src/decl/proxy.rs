use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Ident, Path, Result, Visibility,
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

impl Proxy {
    pub fn expand(&self, extern_trait: &Path) -> TokenStream {
        let Proxy { attrs, vis, ident } = self;

        quote! {
            #(#attrs)*
            #[repr(transparent)]
            #vis struct #ident(#extern_trait::Repr);

            unsafe impl #extern_trait::IntRegRepr for #ident {}
        }
    }
}

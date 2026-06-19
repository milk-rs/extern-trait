use std::{fmt::Display, str::FromStr};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    LitInt,
    parse::{Parse, ParseStream},
};

/// A [LitInt] which has been parsed into an actual integer
/// using [`LitInt::base10_parse`].
///
/// The advantage over a plain [`LitInt`] is that conversion to `value` will never fail.
/// The advantage over just storing an integer is that the original span is preserved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedLitInt<T> {
    pub literal: LitInt,
    pub value: T,
}
impl<T> ToTokens for ParsedLitInt<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.literal.to_tokens(tokens)
    }
}
impl<T: FromStr> Parse for ParsedLitInt<T>
where
    T::Err: Display,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal = input.parse::<LitInt>()?;
        let value = literal.base10_parse()?;
        Ok(ParsedLitInt { literal, value })
    }
}

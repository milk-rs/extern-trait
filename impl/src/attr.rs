// Adapted from https://github.com/dtolnay/linkme/blob/1bb21a8f0f1f1df0301e7df06c435940d05c2447/impl/src/attr.rs

use syn::parse::{Error, Result};
use syn::{Attribute, Path, parse_quote};

// #[extern_trait(crate = path::to::extern_trait)]
pub(crate) fn extern_trait_path(attrs: &mut Vec<Attribute>) -> Result<Path> {
    let mut extern_trait_path = None;
    let mut errors: Option<Error> = None;

    attrs.retain(|attr| {
        if !attr.path().is_ident("extern_trait") {
            return true;
        }
        if let Err(err) = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                if extern_trait_path.is_some() {
                    return Err(meta.error("duplicate extern_trait crate attribute"));
                }
                let path = meta.value()?.call(Path::parse_mod_style)?;
                extern_trait_path = Some(path);
                Ok(())
            } else {
                Err(meta.error("unsupported extern_trait attribute"))
            }
        }) {
            match &mut errors {
                None => errors = Some(err),
                Some(errors) => errors.combine(err),
            }
        }
        false
    });

    match errors {
        None => Ok(extern_trait_path.unwrap_or_else(|| parse_quote!(::extern_trait))),
        Some(errors) => Err(errors),
    }
}

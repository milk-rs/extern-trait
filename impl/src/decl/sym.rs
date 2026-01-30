#![allow(unused)]

use std::env::var;

use proc_macro::Span;

fn hash(string: &str) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    string.hash(&mut hasher);
    hasher.finish()
}

// Code adapted from https://github.com/knurling-rs/defmt/blob/023449c35f68b9dfc2e00437e47353755d5189ef/macros/src/construct.rs
fn crate_local_disambiguator() -> u64 {
    // We want a deterministic, but unique-per-macro-invocation identifier. For that we
    // hash the call site `Span`'s debug representation, which contains a counter that
    // should disambiguate macro invocations within a crate.
    hash(&format!("{:?}", Span::call_site()))
}

#[derive(Debug, Clone)]
pub struct Symbol {
    extern_trait: String,
    package: String,
    version: String,
    crate_name: String,
    package_disambiguator: u64,
    trait_name: String,
    local_disambiguator: u64,
    name: String,
}

impl Symbol {
    pub fn new(trait_name: String) -> Self {
        Self {
            extern_trait: "v0".to_string(),
            package: var("CARGO_PKG_NAME").unwrap_or("<unknown>".to_string()),
            version: var("CARGO_PKG_VERSION").unwrap_or("<unknown>".to_string()),
            crate_name: var("CARGO_CRATE_NAME").unwrap_or("<unknown>".to_string()),
            package_disambiguator: hash(var("CARGO_MANIFEST_PATH").as_deref().unwrap_or_default()),
            trait_name,
            local_disambiguator: crate_local_disambiguator(),
            name: String::new(),
        }
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
}

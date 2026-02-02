# AGENTS.md - Guidelines for AI Coding Agents

This document provides essential information for AI agents working on the `extern-trait` crate.

## Project Overview

`extern-trait` is a Rust procedural macro crate that generates proxy types for trait method calls across linker boundaries using symbol-based linking. It acts as a "static vtable" - method calls are resolved at link time with zero heap allocation.

**Crate Structure:**
- `extern-trait` (root): Main crate exposing the macro and `Repr` type
- `extern-trait-impl` (`impl/`): Proc-macro implementation crate

## Build/Lint/Test Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Run a single test by name
cargo test test_resource
cargo test test_atomic

# Run tests in a specific file
cargo test --test arc
cargo test --test static

# Run tests with output
cargo test -- --nocapture

# Format (requires nightly for unstable features)
cargo +nightly fmt
cargo +nightly fmt --check

# Lint
cargo clippy
cargo clippy --all-targets
```

## Code Style Guidelines

### Edition and Features

- **Edition**: 2024
- **No-std**: Main crate is `#![no_std]` compatible
- Uses nightly rustfmt features

### Import Organization

Imports use `group_imports = "StdExternalCrate"` and `imports_granularity = "Crate"`:

```rust
// 1. Standard library
use std::fmt::Debug;

// 2. External crates
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Ident, Result};

// 3. Crate-local
use crate::args::DeclArgs;
```

### Naming Conventions

- **Types/Traits**: PascalCase (`VerifiedSignature`, `SelfKind`)
- **Functions/Methods**: snake_case (`try_new`, `arg_names`)
- **Constants**: SCREAMING_SNAKE_CASE (`TRAITS`, `GLOBAL`)
- **Proxy types**: Suffix with `Proxy` (`HelloProxy`, `ResourceProxy`)
- **Implementation types**: Suffix with `Impl` (`HelloImpl`, `AtomicImpl`)

### Error Handling

Use `syn::Result<T>` for proc-macro functions. Create errors with `Error::new_spanned()`:

```rust
pub fn expand(args: DeclArgs, input: ItemTrait) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            input.generics,
            "#[extern_trait] may not have generics",
        ));
    }
    // ...
}
```

### Safety and Unsafe Code

- Document safety with `// SAFETY:` comments
- Use `unsafe extern "Rust"` blocks for FFI declarations
- Use `#[unsafe(export_name = ...)]` for symbol exports

```rust
// SAFETY: We asserted size_of::<T>() <= size_of::<Repr>()
unsafe {
    core::ptr::write(repr.as_mut_ptr().cast::<T>(), value);
    repr.assume_init()
}
```

### Testing Patterns

Tests are in `tests/` as integration tests. Use modules to isolate implementations:

```rust
#[extern_trait(AtomicProxy)]
trait Atomic {
    fn new(v: i32) -> Self;
}

mod atomic_impl {
    use super::*;
    struct AtomicImpl(AtomicI32);
    
    #[extern_trait]
    impl Atomic for AtomicImpl { /* ... */ }
}

#[test]
fn test_atomic() {
    let atomic = AtomicProxy::new(0);
    assert_eq!(atomic.get(), 0);
}
```

### Key Constraints

1. **Size**: Implementation types must fit in `Repr` (16 bytes on 64-bit)
2. **No generics**: Traits cannot have generic parameters
3. **Methods only**: No associated types or constants
4. **FFI-compatible**: No `const`, `async`, or generic methods
5. **Self types**: Only `Self`, `&Self`, `&mut Self`, `*const Self`, `*mut Self`

### Module Organization

```
extern-trait/
├── src/lib.rs          # Main crate: Repr type, re-exports
├── impl/src/
│   ├── lib.rs          # Macro entry point
│   ├── args.rs         # Argument parsing
│   ├── imp.rs          # impl block expansion
│   └── decl/           # Trait declaration expansion
│       ├── mod.rs
│       ├── sig.rs      # Signature verification
│       ├── sym.rs      # Symbol generation
│       └── supertraits.rs
└── tests/              # Integration tests
```

### Common Patterns

- Use `let-else` for early returns: `let Some(x) = opt else { return }`
- Use `if let` with `&&` guards for complex conditionals
- Prefer `impl Iterator` over collecting into Vec
- Use `format_ident!` for programmatic identifier creation
- Separate parsing from expansion logic
- Use `quote!` for code generation, `parse_quote!` for inline syn types

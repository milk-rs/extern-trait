# AGENTS.md - Guidelines for AI Coding Agents

This document provides essential information for AI agents working on the `extern-trait` crate.

## Project Overview

`extern-trait` is a Rust procedural macro that creates opaque proxy types for traits using link-time static dispatch instead of dynamic dispatch (`dyn Trait`). Method calls are resolved through exported linker symbols, enabling cross-crate abstraction without vtables or heap allocation.

**Workspace Structure:**
- `extern-trait` (root): Main crate with `#[extern_trait]` macro and `Repr` type
- `extern-trait-impl` (`impl/`): Proc-macro implementation

## Build/Lint/Test Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Run a single test by name
cargo test test_resource

# Run tests in a specific file
cargo test --test arc
cargo test --test reflect

# Format check (CI uses stable)
cargo fmt --all --check

# Lint (CI runs with -D warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Documentation
cargo doc --all-features --no-deps
```

## Code Style Guidelines

### Edition and Features

- **Edition**: 2024 (uses `unsafe extern`, let-else, const blocks)
- **No-std**: Main crate is `#![no_std]` compatible
- **Rustfmt**: `style_edition = "2024"`, `group_imports = "StdExternalCrate"`

### Import Organization

```rust
// 1. Standard library
use core::mem::MaybeUninit;

// 2. External crates
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Ident, Result, parse_quote};

// 3. Crate-local
use self::sig::VerifiedSignature;
use crate::args::DeclArgs;
```

### Naming Conventions

| Element | Convention | Examples |
|---------|------------|----------|
| Types/Structs | PascalCase | `VerifiedSignature`, `SelfKind` |
| Functions | snake_case | `try_new`, `from_value` |
| Constants | SCREAMING_SNAKE_CASE | `TRAITS`, `TYPEID` |
| Proxy types | Suffix `Proxy` | `HelloProxy`, `ResourceProxy` |
| Impl types | Suffix `Impl` | `HelloImpl`, `AtomicImpl` |

### Error Handling

Use `syn::Result<T>` with `Error::new_spanned()`:
```rust
if sig.constness.is_some() {
    return Err(Error::new_spanned(sig.constness, "..."));
}
```

### Unsafe Code

**Always use `// SAFETY:` comments:**
```rust
// SAFETY: We asserted size_of::<T>() <= size_of::<Repr>()
unsafe { core::ptr::write(repr.as_mut_ptr().cast::<T>(), value) }
```

**FFI patterns (Rust 2024):**
```rust
unsafe extern "Rust" {
    #[link_name = "..."]
    unsafe fn method(...);
}
#[unsafe(export_name = "...")]
fn exported_method(...) { }
```

### Testing Patterns

Tests in `tests/` as integration tests. Use modules to isolate implementations:

```rust
#[extern_trait(AtomicProxy)]
trait Atomic { fn new(v: i32) -> Self; }

mod atomic_impl {
    use super::*;
    struct AtomicImpl(AtomicI32);
    #[extern_trait]
    impl Atomic for AtomicImpl { /* ... */ }
}

#[test]
fn test_atomic() {
    let atomic = AtomicProxy::new(0);
}
```

## Key Constraints

1. **Size**: Impl types must fit in `Repr` (16 bytes on 64-bit)
2. **No generics**: Traits cannot have generic parameters
3. **Methods only**: No associated types or constants
4. **FFI-compatible**: No `const`, `async`, or generic methods
5. **Self types**: Only `Self`, `&Self`, `&mut Self`, `*const Self`, `*mut Self`

## Module Organization

```
extern-trait/
├── src/lib.rs              # Repr type, re-exports proc-macro
├── impl/src/
│   ├── lib.rs              # Macro entry point
│   ├── args.rs             # Argument parsing
│   ├── imp.rs              # impl block expansion
│   └── decl/
│       ├── mod.rs          # Trait declaration expansion
│       ├── sig.rs          # Signature verification
│       ├── sym.rs          # Symbol name generation
│       └── supertraits.rs  # Send, Sync, Clone, Debug, etc.
└── tests/                  # Integration tests
```

## Common Patterns

```rust
// Let-else for early returns
let Some((_, trait_, _)) = &input.trait_ else { return Err(...) };

// Quote macros for code generation
quote! { ... }              // Generate TokenStream
parse_quote!(#ident)        // Create syn types
format_ident!("{}Proxy", x) // Programmatic identifiers
```

Prefer iterators over collecting into Vec.

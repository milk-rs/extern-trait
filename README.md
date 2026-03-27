# `#[extern_trait]`

[![Crates.io](https://img.shields.io/crates/v/extern-trait?style=flat-square&logo=rust)](https://crates.io/crates/extern-trait)
[![docs.rs](https://img.shields.io/docsrs/extern-trait?style=flat-square&logo=docs.rs)](https://docs.rs/extern-trait)

Opaque types for traits using link-time static dispatch instead of `dyn Trait`.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
extern-trait = "0.1"
```

## Motivation

In modular systems like OS kernels, a common pattern emerges: crate A needs to call functionality that crate B provides, but A cannot depend on B (to avoid circular dependencies or to keep A generic). Examples include:

- A logging crate that needs platform-specific console output
- A filesystem crate that needs a block device driver
- A scheduler that needs architecture-specific context switching

The traditional solution is `Box<dyn Trait>`, but this has drawbacks:

- **Heap allocation** for every trait object
- **Vtable indirection** on every method call
- **Runtime overhead** that may be unacceptable in performance-critical code

`#[extern_trait]` solves this by acting as a **static vtable** - method calls are resolved at link time rather than runtime, with zero heap allocation and no pointer indirection.

## How it Works

1. **Proxy generation**: The macro creates a fixed-size proxy struct that stores the implementation value inline
2. **Symbol export**: Each trait method is exported as a linker symbol from the implementation crate
3. **Symbol linking**: The proxy calls these symbols, which the linker resolves to the actual implementation

Think of it as compile-time monomorphization deferred to link time.

The proxy uses a fixed-size representation:

```rust
#[repr(C)]
struct Repr(*const (), *const ());
```

This is two pointers in size (16 bytes on 64-bit, 8 bytes on 32-bit), storing the implementation value directly - no heap allocation or pointer indirection is added by the macro.

## Example

```rust
# use extern_trait::extern_trait;
// In crate A
/// A Hello trait.
#[extern_trait(
    /// A proxy type for [`Hello`].
    pub(crate) HelloProxy
)]
pub trait Hello {
    fn new(num: i32) -> Self;
    fn hello(&self);
}

let v = HelloProxy::new(42);
v.hello();

// In crate B
struct HelloImpl(i32);

#[extern_trait]
impl Hello for HelloImpl {
    fn new(num: i32) -> Self {
        Self(num)
    }

    fn hello(&self) {
        println!("Hello, {}", self.0)
    }
}
```

<details>

<summary>View generated code</summary>

```rust
// In crate A
/// A Hello trait.
pub trait Hello {
    fn new(num: i32) -> Self;
    fn hello(&self);
}

/// A proxy type for [`Hello`].
#[repr(transparent)]
pub(crate) struct HelloProxy(::extern_trait::Repr);

impl Hello for HelloProxy {
    fn new(_0: i32) -> Self {
        unsafe extern "Rust" {
            #[link_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"new\" }"]
            unsafe fn new(_: i32) -> HelloProxy;

        }
        unsafe { new(_0) }
    }

    fn hello(&self) {
        unsafe extern "Rust" {
            #[link_name = "Symbol { 
```

</details>

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
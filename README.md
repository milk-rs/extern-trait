# `#[extern_trait]`

[![Crates.io](https://img.shields.io/crates/v/extern-trait?style=flat-square&logo=rust)](https://crates.io/crates/extern-trait)
[![docs.rs](https://img.shields.io/docsrs/extern-trait?style=flat-square&logo=docs.rs)](https://docs.rs/extern-trait)

Generate proxy types that forward trait method calls across linker boundaries using symbol-based linking.

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
            #[link_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"hello\" }"]
            unsafe fn hello(_: &HelloProxy);
        }
        unsafe { hello(self) }
    }
}

impl Drop for HelloProxy {
    fn drop(&mut self) {
        unsafe extern "Rust" {
            #[link_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"drop\" }"]
            unsafe fn drop(this: *mut HelloProxy);
        }
        unsafe { drop(self) }
    }
}

// In crate B
struct HelloImpl(i32);

impl Hello for HelloImpl {
    fn new(num: i32) -> Self {
        Self(num)
    }

    fn hello(&self) {
        println!("Hello, {}", self.0)
    }
}

const _: () = {
    assert!(
        ::core::mem::size_of::<HelloImpl>() <= ::core::mem::size_of::<::extern_trait::Repr>(),
        "HelloImpl is too large to be used with #[extern_trait]"
    );
};

const _: () = {
    #[unsafe(export_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"new\" }")]
    fn new(_0: i32) -> ::extern_trait::Repr {
        ::extern_trait::Repr::from_value(<HelloImpl as Hello>::new(_0))
    }
};
const _: () = {
    #[unsafe(export_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"hello\" }")]
    fn hello(_0: &HelloImpl) {
        <HelloImpl as Hello>::hello(_0)
    }
};
const _: () = {
    #[unsafe(export_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"drop\" }")]
    unsafe fn drop(this: &mut HelloImpl) {
        unsafe { ::core::ptr::drop_in_place(this) };
    }
};
```

</details>

## Trait Restrictions

- No generics on the trait itself
- Only methods allowed (no associated types or constants)
- Methods must be FFI-compatible: no `const`, `async`, or generic parameters
- `Self` in signatures must be one of: `Self`, `&Self`, `&mut Self`, `*const Self`, `*mut Self`

## Size Constraint

The implementation type must fit within `Repr`, which is two pointers in size:

| Platform | `Repr` size | Max impl size |
| -------- | ----------- | ------------- |
| 64-bit   | 16 bytes    | 16 bytes      |
| 32-bit   | 8 bytes     | 8 bytes       |

This constraint is checked at compile time. Types that fit include:

- Pointer-sized types: `Box<T>`, `Arc<T>`, `&T`, `*const T`
- Small structs: up to two `usize` fields
- Primitives: integers, floats, bools

For larger types, wrap them in `Box`.

## Supertraits

An `#[extern_trait]` can have supertraits, and the macro will automatically forward their implementations to the proxy type.

**Supported supertraits:**

| Marker traits | Standard traits |
| ------------- | --------------- |
| `Send`        | `Clone`         |
| `Sync`        | `Default`       |
| `Sized`       | `Debug`         |
| `Unpin`       | `AsRef<T>`      |
| `Copy`        | `AsMut<T>`      |

```rust
use std::fmt::Debug;
use extern_trait::extern_trait;

#[extern_trait(ResourceProxy)]
trait Resource: Send + Sync + Clone + Debug {
    fn new() -> Self;
}
```

## Re-exporting / Renaming

By default, the macro references `::extern_trait`. If you re-export or rename the crate, use the `crate` attribute to specify the correct path:

```rust
use ::extern_trait as my_extern_trait;

use my_extern_trait::extern_trait;

// Specify the path when defining a trait
#[extern_trait(crate = my_extern_trait, MyProxy)]
trait MyTrait {
    fn new() -> Self;
}

struct MyImpl;

// Also specify the path when implementing
#[extern_trait(crate = my_extern_trait)]
impl MyTrait for MyImpl {
    fn new() -> Self { MyImpl }
}
```

This is also necessary if you rename the dependency in `Cargo.toml`:

```toml
[dependencies]
my_extern_trait = { package = "extern-trait", version = "..." }
```

## Internals

### Why Two Pointers?

The `Repr` type is two pointers in size based on a key observation: **most calling conventions pass structs up to two registers by value in registers, not on the stack**.

On x86_64, ARM64, RISC-V, and other common architectures, a two-pointer struct is passed and returned in two registers (e.g., `rdi`+`rsi`/`rax`+`rdx` on x86_64, `x0`+`x1` on ARM64). This means:

- **No memory traffic**: Values stay in registers across function calls
- **Zero-cost conversion**: `Repr::from_value` and `Repr::into_value` compile to nothing

For example, on x86_64:

```asm
; from_value<Box<T>> - the Box pointer is already in rdi, just move to rax
mov     rax, rdi
ret
```

On architectures that don't pass two-pointer structs in registers, this still works correctly - just with a small memory copy instead of pure register operations. The design prioritizes the common case while remaining portable.

### What Fits in `Repr`?

| Type                          | Size (64-bit) | Fits?         |
| ----------------------------- | ------------- | ------------- |
| `Box<T>`, `Arc<T>`, `Rc<T>`   | 8 bytes       | ✓             |
| `&T`, `*const T`              | 8 bytes       | ✓             |
| `(usize, usize)`              | 16 bytes      | ✓             |
| `&[T]`, `&str` (fat pointers) | 16 bytes      | ✓             |
| `String`, `Vec<T>`            | 24 bytes      | ✗ (use `Box`) |

Two pointers is the sweet spot: it covers fat pointers, smart pointers, and small structs - the types you'd typically use to implement a trait.

## Credits

This crate is inspired by [crate_interface](https://github.com/arceos-org/crate_interface).

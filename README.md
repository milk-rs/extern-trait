# `#[extern_trait]`

[![Crates.io](https://img.shields.io/crates/v/extern-trait?style=flat-square&logo=rust)](https://crates.io/crates/extern-trait)
[![docs.rs](https://img.shields.io/docsrs/extern-trait?style=flat-square&logo=docs.rs)](https://docs.rs/extern-trait)

Generate proxy types that forward trait method calls across linker boundaries using symbol-based linking.

This enables Rust-to-Rust dynamic dispatch without `dyn` trait objects - useful for OS development and other scenarios where trait implementations live in separate crates that are linked together.

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

unsafe impl extern_trait::IntRegRepr for HelloImpl {}

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

unsafe impl ::extern_trait::IntRegRepr for HelloProxy {}

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

unsafe impl extern_trait::IntRegRepr for HelloImpl {}

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
        ::core::mem::size_of::<HelloImpl>() <= ::core::mem::size_of::<::extern_trait::Repr>() * 2,
        "HelloImpl is too large to be used with #[extern_trait]"
    );
};

const _: () = {
    #[unsafe(export_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"new\" }")]
    fn new(_0: i32) -> ::extern_trait::Repr {
        ::extern_trait::IntRegRepr::into_repr({ <HelloImpl as Hello>::new(_0) })
    }
};
const _: () = {
    #[unsafe(export_name = "Symbol { ..., trait_name: \"Hello\", ..., name: \"hello\" }")]
    fn hello(_0: &HelloImpl) {
        ({ <HelloImpl as Hello>::hello(_0) })
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

## Implementor Requirements

The implementation type must implement [`IntRegRepr`](https://docs.rs/extern-trait/latest/extern_trait/trait.IntRegRepr.html), which requires the type to be passed in **integer registers only** when used in `extern "C"` calls.

Most pointer-like types (references, `Box`, `Arc`, etc.) already implement `IntRegRepr`.
For custom types, you must `unsafe impl` the trait - see the documentation for details.

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

## Credits

This crate is inspired by [crate_interface](https://github.com/arceos-org/crate_interface).

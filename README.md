# `#[extern_trait]`

[![Crates.io](https://img.shields.io/crates/v/extern-trait?style=flat-square&logo=rust)](https://crates.io/crates/extern-trait)
[![docs.rs](https://img.shields.io/docsrs/extern-trait?style=flat-square&logo=docs.rs)](https://docs.rs/extern-trait)

Generate an opaque type for a trait to forward to a foreign implementation.

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

unsafe impl extern_trait::ExternSafe for HelloImpl {}

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

This will generate the following code (adapted):

<details>

<summary>View code</summary>

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

unsafe impl ::extern_trait::ExternSafe for HelloProxy {}

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

unsafe impl extern_trait::ExternSafe for HelloImpl {}

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
        ::extern_trait::ExternSafe::into_repr({ <HelloImpl as Hello>::new(_0) })
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

## Supertraits

An `#[extern_trait]` may have supertraits to forward more trait implementations. The currently supported traits are:
- `Send`/`Sync`
- `AsRef`
- *TODO: support more*

```rust
use extern_trait::extern_trait;

#[extern_trait(FooProxy)]
trait Foo: Send {
    fn foo();
}
```

## Restrictions

For the trait:
- It may not have generics.
- It may only contain methods, not associated types or constants.
- Its methods have to be compatible with [FFI](https://doc.rust-lang.org/reference/items/external-blocks.html#functions), i.e. no `const`/`async`/type parameters/const parameters
- If `Self` type appears in any location (including the method receiver), it has to be one of the following forms: **`Self`/`&Self`/`&mut Self`/`*const Self`/`*mut Self`**.
  - Currently `Self` can not be used as parameter type, but maybe supported in the future.

For the implementor: The type must be able to pass through two general registers in calling conventions. That basically requires the following things:
- Smaller than two general registers (e.g. **<= 16 bytes** on 64-bit architectures)
- Do not use floating point registers unless using soft-float ABI

`#[extern_trait]` automatically checked the first requirement, but there are no way to check the second one. So `#[extern_trait]` is required to be **`unsafe`** and implementor must guarantee that their type satisfy all the requirements.

This also require the ABI to be able to pass value in two general registers, so not all architectures and platforms are supported.
- *TODO: support table*

## Credits

This crate is heavily inspired by [crate_interface](https://github.com/arceos-org/crate_interface), as the original starting point was to solve the problem that crate_interface cannot pass opaque types.

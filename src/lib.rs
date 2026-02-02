#![no_std]
#![doc = include_str!("../README.md")]

pub use extern_trait_impl::*;

#[doc(hidden)]
#[repr(C)]
pub struct Repr(*const (), *const ());

extern "C" fn reflect<T>(this: T) -> T {
    this
}

/// A type with an **integer-register-compatible** representation.
///
/// Types implementing this trait can be safely passed across linker symbol boundaries
/// in `extern "C"` function calls. This is the core requirement for types used with
/// `#[extern_trait]`.
///
/// # Requirements
///
/// A type may implement `IntRegRepr` if and only if it satisfies **both** of the following:
///
/// 1. **Size constraint**: The type must fit within two integer registers
///    (i.e., `size_of::<T>() <= 2 * size_of::<usize>()`).
///
/// 2. **Integer register constraint**: When passed as an argument or return value
///    in an `extern "C"` function, the type must use **integer registers only** -
///    not floating-point registers. This means types containing floating-point fields
///    (`f32`, `f64`) are **only** allowed on soft-float ABIs where floats are passed
///    in integer registers.
///
/// The first constraint is checked at compile time by `#[extern_trait]`. The second
/// constraint **cannot** be verified automatically - implementors must ensure their
/// type satisfies this on all target platforms.
///
/// # Why integer registers?
///
/// The `#[extern_trait]` macro generates proxy types that forward method calls across
/// linker symbol boundaries via `extern "C"` functions. Values are passed by transmuting
/// them through an intermediate representation defined as:
///
/// ```ignore
/// #[repr(C)]
/// struct Repr(*const (), *const ());
/// ```
///
/// Since `Repr` is a pair of pointers, it is always passed in **two integer registers**.
/// For the transmute to be valid, your type must occupy the same integer registers -
/// if your type uses floating-point registers instead, the bit pattern will not be
/// preserved correctly across the call boundary.
///
/// # Implementing this trait
///
/// Most common types already have implementations provided by this crate:
/// - Primitives: integers, `bool`, `()`, atomics
/// - Pointers: `*const T`, `*mut T`, `&T`, `&mut T`, [`NonNull<T>`](core::ptr::NonNull)
/// - Smart pointers (with `alloc` feature): `Box<T>`, `Arc<T>`, `Rc<T>`
/// - Wrappers: [`Pin<T>`](core::pin::Pin), [`MaybeUninit<T>`](core::mem::MaybeUninit),
///   [`Cell<T>`](core::cell::Cell), [`UnsafeCell<T>`](core::cell::UnsafeCell)
///
/// For custom types, use `unsafe impl`:
///
/// ```
/// # use extern_trait::IntRegRepr;
/// struct MyHandle(*const ());
///
/// // SAFETY: MyHandle is pointer-sized and passed in an integer register.
/// unsafe impl IntRegRepr for MyHandle {}
/// ```
///
/// # Safety
///
/// Implementing this trait incorrectly causes **undefined behavior**. When in doubt,
/// wrap your data in `Box` (which is always `IntRegRepr`).
///
/// # Hidden methods
///
/// This trait contains hidden methods used internally by `#[extern_trait]`. Do not call
/// or override these methods unless you fully understand the ABI implications.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used with `#[extern_trait]`",
    label = "`{Self}` does not implement `IntRegRepr`"
)]
pub unsafe trait IntRegRepr: Sized {
    #[doc(hidden)]
    fn into_repr(self) -> Repr {
        let transmute = unsafe {
            core::mem::transmute::<*const (), extern "C" fn(Self) -> Repr>(
                reflect::<Self> as *const (),
            )
        };
        transmute(self)
    }

    #[doc(hidden)]
    fn from_repr(repr: Repr) -> Self {
        let transmute = unsafe {
            core::mem::transmute::<*const (), extern "C" fn(Repr) -> Self>(
                reflect::<Self> as *const (),
            )
        };
        transmute(repr)
    }
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use typeid::ConstTypeId;
}

mod impls;

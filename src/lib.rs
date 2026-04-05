#![no_std]
#![doc = include_str!("../README.md")]

pub use extern_trait_impl::*;

/// Opaque representation used to store implementation types in proxy structs.
///
/// This type is two pointers in size, which means implementation types must be
/// at most `2 * size_of::<usize>()` bytes (16 bytes on 64-bit, 8 bytes on 32-bit).
///
/// The size constraint is checked at compile time.
#[doc(hidden)]
#[repr(C)]
pub struct Repr(
    // This strange form is used to make this type `!Send`, `!Sync`, `!Unpin`, `!UnwindSafe`, `!RefUnwindSafe` and `!Freeze` without using any unstable features.
    *mut (),
    &'static mut (),
    core::cell::UnsafeCell<()>,
    core::marker::PhantomPinned,
);

const _: () = assert!(size_of::<Repr>() == size_of::<usize>() * 2);

impl Repr {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_value<T: Sized>(value: T) -> Self {
        const { assert!(size_of::<T>() <= size_of::<Repr>()) };
        let mut repr = core::mem::MaybeUninit::<Repr>::zeroed();
        // SAFETY: We just asserted that size_of::<T>() <= size_of::<Repr>()
        unsafe {
            core::ptr::write(repr.as_mut_ptr().cast::<T>(), value);
            repr.assume_init()
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn into_value<T: Sized>(self) -> T {
        const { assert!(size_of::<T>() <= size_of::<Repr>()) };
        // SAFETY: We require that size_of::<T>() <= size_of::<Repr>(),
        // and the caller ensures the Repr was created from a valid T.
        unsafe { core::ptr::read((&self as *const Repr).cast::<T>()) }
    }
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use typeid::ConstTypeId;
}

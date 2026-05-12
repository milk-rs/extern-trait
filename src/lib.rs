#![no_std]
#![doc = include_str!("../README.md")]

pub use extern_trait_impl::*;

/// Opaque representation used to store implementation types in proxy structs.
///
/// This type is two pointers in size, which means implementation types must be
/// at most `2 * size_of::<usize>()` bytes (16 bytes on 64-bit, 8 bytes on 32-bit).
///
/// The size and alignment constraints are checked at compile time.
#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Repr(
    *mut (),
    *mut (),
    // make this type `!Send + !Sync + !Unpin + !UnwindSafe + !RefUnwindSafe + !Freeze`
    core::marker::PhantomData<(
        &'static mut (),
        core::cell::UnsafeCell<()>,
        core::marker::PhantomPinned,
    )>,
);

const _: () = assert!(size_of::<Repr>() == size_of::<usize>() * 2);

impl Repr {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_value<T: Sized>(value: T) -> Self {
        const { assert!(size_of::<T>() <= size_of::<Repr>()) };
        const { assert!(align_of::<T>() <= align_of::<Repr>()) };
        let mut repr = core::mem::MaybeUninit::<Repr>::zeroed();
        // SAFETY: We just asserted that T fits in Repr and does not require stricter alignment.
        unsafe {
            core::ptr::write(repr.as_mut_ptr().cast::<T>(), value);
            repr.assume_init()
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn into_value<T: Sized>(self) -> T {
        const { assert!(size_of::<T>() <= size_of::<Repr>()) };
        const { assert!(align_of::<T>() <= align_of::<Repr>()) };
        // SAFETY: We require that T fits in Repr and does not require stricter alignment,
        // and the caller ensures the Repr was created from a valid T.
        unsafe { core::ptr::read((&self as *const Repr).cast::<T>()) }
    }
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use typeid::ConstTypeId;
}

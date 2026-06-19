#![no_std]
#![doc = include_str!("../README.md")]

pub use extern_trait_impl::*;

/// Opaque representation, with the default size [`DEFAULT_PTR_COUNT`].
pub type Repr = ReprN<{ DEFAULT_PTR_COUNT }>;

/// Opaque representation used to store implementation types in proxy structs
///
/// This type is parameterized by the `PTR_COUNT`, which determines the type size in terms of pointers.
/// This means implementation types must be
/// at most `PTR_COUNT * size_of::<usize>()` bytes.
/// By default, `PTR_COUNT = 2` (see [`DEFAULT_PTR_COUNT`]).
/// This is 16 bytes on 64-bit, 8 bytes on 32-bit.
///
/// The size constraint is checked at compile time.
#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ReprN<const PTR_COUNT: usize>(
    [*mut (); PTR_COUNT],
    // make this type `!Send + !Sync + !Unpin + !UnwindSafe + !RefUnwindSafe + !Freeze`
    core::marker::PhantomData<(
        &'static mut (),
        core::cell::UnsafeCell<()>,
        core::marker::PhantomPinned,
    )>,
);

/// The default size of a [`ReprN`], if not otherwise specified.
pub const DEFAULT_PTR_COUNT: usize = 2;
const _: () = {
    assert!(size_of::<ReprN<DEFAULT_PTR_COUNT>>() == size_of::<usize>() * DEFAULT_PTR_COUNT);
    assert!(size_of::<ReprN<0>>() == 0);
    assert!(size_of::<ReprN<1>>() == size_of::<usize>());
    assert!(size_of::<ReprN<2>>() == size_of::<usize>() * 2);
    assert!(size_of::<ReprN<3>>() == size_of::<usize>() * 3);
};

impl<const PTR_COUNT: usize> ReprN<PTR_COUNT> {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_value<T: Sized>(value: T) -> Self {
        const {
            assert!(size_of::<Self>() == size_of::<usize>() * PTR_COUNT);
            assert!(size_of::<T>() <= size_of::<Self>());
        }
        let mut repr = core::mem::MaybeUninit::<Self>::zeroed();
        // SAFETY: We just asserted that size_of::<T>() <= size_of::<Repr>()
        unsafe {
            core::ptr::write(repr.as_mut_ptr().cast::<T>(), value);
            repr.assume_init()
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn into_value<T: Sized>(self) -> T {
        const { assert!(size_of::<T>() <= size_of::<Self>()) };
        // SAFETY: We require that size_of::<T>() <= size_of::<Repr>(),
        // and the caller ensures the Repr was created from a valid T.
        unsafe { core::ptr::read((&self as *const Self).cast::<T>()) }
    }
}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use typeid::ConstTypeId;
}

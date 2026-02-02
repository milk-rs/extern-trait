#![no_std]
#![doc = include_str!("../README.md")]

pub use extern_trait_impl::*;

#[doc(hidden)]
#[repr(C)]
pub struct Repr(*const (), *const ());

extern "C" fn reflect<T>(this: T) -> T {
    this
}

/// TODO
/// # Safety
/// TODO
pub unsafe trait ExternSafe: Sized {
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

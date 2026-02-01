#![no_std]
#![doc = include_str!("../README.md")]

pub use extern_trait_impl::*;

/// TODO
/// # Safety
/// TODO
pub unsafe trait ExternSafe {}

#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use typeid::ConstTypeId;
}

mod impls;

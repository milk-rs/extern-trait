use core::{
    cell::{Cell, RefCell, UnsafeCell},
    mem::MaybeUninit,
    num::NonZero,
    pin::Pin,
    ptr::NonNull,
    sync::atomic::*,
};

use super::ExternSafe;

macro_rules! impl_extern_safe {
    ($($t:ty),*) => {
        $(
            unsafe impl ExternSafe for $t {}
        )*
    };
}

impl_extern_safe!(
    (),
    bool,
    // no char!
    u8,
    u16,
    u32,
    usize,
    i8,
    i16,
    i32,
    isize,
    NonZero<u8>,
    NonZero<u16>,
    NonZero<u32>,
    NonZero<usize>,
    NonZero<i8>,
    NonZero<i16>,
    NonZero<i32>,
    NonZero<isize>,
    NonZero<char>,
    AtomicBool,
    AtomicU8,
    AtomicU16,
    AtomicU32,
    AtomicUsize,
    AtomicI8,
    AtomicI16,
    AtomicI32,
    AtomicIsize
);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_extern_safe!(u64, i64, NonZero<u64>, NonZero<i64>, AtomicU64, AtomicI64);

#[cfg(target_pointer_width = "64")]
impl_extern_safe!(u128, i128, NonZero<u128>, NonZero<i128>);

#[cfg(any(
    target_feature = "soft-float",
    target_abi = "softfloat",
    target_abi = "eabi"
    // TODO: handle riscv
))]
impl_extern_safe!(f32, f64);

unsafe impl<T: ?Sized> ExternSafe for *const T {}
unsafe impl<T: ?Sized> ExternSafe for *mut T {}
unsafe impl<T: ?Sized> ExternSafe for NonNull<T> {}
unsafe impl<T: ?Sized> ExternSafe for &T {}
unsafe impl<T: ?Sized> ExternSafe for &mut T {}

unsafe impl<T: ExternSafe, const N: usize> ExternSafe for [T; N] {}
unsafe impl<T: ExternSafe> ExternSafe for [T] {}

unsafe impl<T: ExternSafe> ExternSafe for Pin<T> {}
unsafe impl<T: ExternSafe> ExternSafe for MaybeUninit<T> {}
unsafe impl<T: ExternSafe> ExternSafe for Option<T> {}
unsafe impl<T: ExternSafe, E: ExternSafe> ExternSafe for Result<T, E> {}

unsafe impl<T: ExternSafe + ?Sized> ExternSafe for UnsafeCell<T> {}
unsafe impl<T: ExternSafe + ?Sized> ExternSafe for Cell<T> {}
unsafe impl<T: ExternSafe + ?Sized> ExternSafe for RefCell<T> {}

#[cfg(feature = "alloc")]
mod alloc_impls {
    extern crate alloc;

    use alloc::{
        boxed::Box,
        collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
        rc::{Rc, Weak as RcWeak},
        string::String,
        sync::{Arc, Weak as ArcWeak},
        vec::Vec,
    };

    use super::*;

    unsafe impl<T: ?Sized> ExternSafe for Box<T> {}
    unsafe impl<T: ?Sized> ExternSafe for Rc<T> {}
    unsafe impl<T: ?Sized> ExternSafe for RcWeak<T> {}
    unsafe impl<T: ?Sized> ExternSafe for Arc<T> {}
    unsafe impl<T: ?Sized> ExternSafe for ArcWeak<T> {}

    // Note: impls below are actually useless; They exceed the size limit.
    // Keep them here for reducing compiler errors when users try to use these types.
    unsafe impl ExternSafe for String {}
    unsafe impl<T> ExternSafe for Vec<T> {}
    unsafe impl<T> ExternSafe for VecDeque<T> {}
    unsafe impl<T> ExternSafe for LinkedList<T> {}
    unsafe impl<T> ExternSafe for BTreeSet<T> {}
    unsafe impl<K, V> ExternSafe for BTreeMap<K, V> {}
    unsafe impl<T> ExternSafe for BinaryHeap<T> {}
}

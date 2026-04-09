use extern_trait::extern_trait;

/// # Safety
/// This trait is safe.
#[extern_trait(UnsafeProxy)]
unsafe trait UnsafeTrait {
    fn make(v: i32) -> Self;
    unsafe fn danger(&self) -> i32;
}

mod imp {
    use super::*;

    struct UnsafeImpl(i32);

    #[extern_trait]
    unsafe impl UnsafeTrait for UnsafeImpl {
        fn make(v: i32) -> Self {
            Self(v)
        }

        unsafe fn danger(&self) -> i32 {
            self.0
        }
    }
}

#[test]
fn unsafe_things() {
    let p = UnsafeProxy::make(10);
    assert_eq!(unsafe { p.danger() }, 10);
}

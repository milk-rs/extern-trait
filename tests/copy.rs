use extern_trait::extern_trait;

#[extern_trait(CopyProxy)]
trait CopyApi: Clone + Copy {
    fn new(v: u8) -> Self;
    fn value(&self) -> u8;
}

mod copy_impl {
    use super::*;

    #[derive(Clone, Copy)]
    struct CopyImpl(u8);

    #[extern_trait]
    impl CopyApi for CopyImpl {
        fn new(v: u8) -> Self {
            Self(v)
        }

        fn value(&self) -> u8 {
            self.0
        }
    }
}

#[test]
fn copy_proxy_is_copy() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<CopyProxy>();

    assert!(!std::mem::needs_drop::<CopyProxy>());

    let a = CopyProxy::new(42);
    let b = a;
    assert_eq!(a.value(), 42);
    assert_eq!(b.value(), 42);
}

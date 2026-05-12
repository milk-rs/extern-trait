use extern_trait::extern_trait;

#[extern_trait(CopyOnlyProxy)]
trait CopyOnly: Copy {
    fn new(value: u8) -> Self;
    fn value(&self) -> u8;
}

mod copy_only_impl {
    use super::*;

    #[derive(Clone, Copy)]
    struct CopyOnlyImpl(u8);

    #[extern_trait]
    impl CopyOnly for CopyOnlyImpl {
        fn new(value: u8) -> Self {
            Self(value)
        }

        fn value(&self) -> u8 {
            self.0
        }
    }
}

fn assert_clone<T: Clone>() {}
fn assert_copy<T: Copy>() {}

fn main() {
    assert_clone::<CopyOnlyProxy>();
    assert_copy::<CopyOnlyProxy>();

    let a = CopyOnlyProxy::new(7);
    let b = a;
    let c = a.clone();

    assert_eq!(a.value(), 7);
    assert_eq!(b.value(), 7);
    assert_eq!(c.value(), 7);
}

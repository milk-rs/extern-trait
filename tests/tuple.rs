use extern_trait::extern_trait;

#[extern_trait(TupleProxy)]
trait Tuple {
    fn new(a: u32, b: u32, c: u32, d: u32) -> Self;
    fn get(&self) -> (u32, u32, u32, u32);
}

mod tuple_impl {
    use super::*;

    struct TupleImpl(u32, u32, u32, u32);

    unsafe impl extern_trait::IntRegRepr for TupleImpl {}

    #[extern_trait]
    impl Tuple for TupleImpl {
        fn new(a: u32, b: u32, c: u32, d: u32) -> Self {
            Self(a, b, c, d)
        }

        fn get(&self) -> (u32, u32, u32, u32) {
            (self.0, self.1, self.2, self.3)
        }
    }
}

#[test]
fn test_tuple() {
    let tuple = TupleProxy::new(1, 2, 3, 4);
    let (a, b, c, d) = tuple.get();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
    assert_eq!(d, 4);
}

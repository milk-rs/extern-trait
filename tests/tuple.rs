use extern_trait::extern_trait;

#[cfg(target_pointer_width = "64")]
type Elem = u32;

#[cfg(target_pointer_width = "32")]
type Elem = u16;

#[extern_trait(TupleProxy)]
trait Tuple {
    fn new(a: Elem, b: Elem, c: Elem, d: Elem) -> Self;
    fn get(&self) -> (Elem, Elem, Elem, Elem);
}

mod tuple_impl {
    use super::*;

    struct TupleImpl(Elem, Elem, Elem, Elem);

    #[extern_trait]
    impl Tuple for TupleImpl {
        fn new(a: Elem, b: Elem, c: Elem, d: Elem) -> Self {
            Self(a, b, c, d)
        }

        fn get(&self) -> (Elem, Elem, Elem, Elem) {
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

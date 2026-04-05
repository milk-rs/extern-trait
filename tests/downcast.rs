use extern_trait::extern_trait;

#[extern_trait(AnyProxy)]
trait AnyValue {
    fn value(&self) -> usize;
    fn add(self, rhs: usize) -> Self;
}

#[derive(Debug)]
struct NumberImpl(usize);

#[derive(Debug)]
struct OtherImpl(usize);

#[extern_trait]
impl AnyValue for NumberImpl {
    fn value(&self) -> usize {
        self.0
    }

    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl AnyValue for OtherImpl {
    fn value(&self) -> usize {
        self.0
    }

    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

#[test]
fn proxy_round_trips_concrete_impl() {
    let mut v = AnyProxy::from_impl(NumberImpl(10));
    assert_eq!(v.value(), 10);

    v.downcast_mut::<NumberImpl>().0 = 20;
    assert_eq!(v.downcast_ref::<NumberImpl>().0, 20);

    let out = v.add(2).into_impl::<NumberImpl>();
    assert_eq!(out.0, 22);
}

#[test]
#[should_panic(expected = "is not an implementation type")]
fn proxy_rejects_from_impl_with_non_exported_impl_type() {
    let _ = AnyProxy::from_impl(OtherImpl(1));
}

#[test]
#[should_panic(expected = "is not an implementation type")]
fn proxy_rejects_incorrect_downcast() {
    let value = AnyProxy::from_impl(NumberImpl(1));
    let _ = value.downcast_ref::<OtherImpl>();
}

#[test]
#[should_panic(expected = "is not an implementation type")]
fn proxy_rejects_into_impl_for_wrong_type() {
    let value = AnyProxy::from_impl(NumberImpl(1));
    let _ = value.into_impl::<OtherImpl>();
}

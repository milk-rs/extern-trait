use extern_trait::extern_trait;

#[extern_trait(AnyProxy)]
trait Any: 'static {}

struct AnyImpl(usize);

unsafe impl extern_trait::ExternSafe for AnyImpl {}

#[extern_trait]
impl Any for AnyImpl {}

#[test]
fn test_any() {
    let any = AnyImpl(42);

    let mut any = AnyProxy::from_impl(any);
    assert_eq!(any.downcast_ref::<AnyImpl>().0, 42);

    any.downcast_mut::<AnyImpl>().0 = 100;
    assert_eq!(any.downcast_ref::<AnyImpl>().0, 100);

    let any = any.into_impl::<AnyImpl>();
    assert_eq!(any.0, 100);
}

use ::extern_trait as my_extern_trait;
use my_extern_trait::extern_trait;

#[extern_trait(crate = my_extern_trait, RenamedProxy)]
trait Renamed {
    fn value(&self) -> i32;
}

struct RenamedImpl(i32);

#[extern_trait(crate = my_extern_trait)]
impl Renamed for RenamedImpl {
    fn value(&self) -> i32 {
        self.0
    }
}

#[test]
fn test_renamed_crate() {
    let proxy = RenamedProxy::from_impl(RenamedImpl(42));
    assert_eq!(proxy.value(), 42);
}

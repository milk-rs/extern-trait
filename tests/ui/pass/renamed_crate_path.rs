use extern_trait::{self as renamed_extern_trait, extern_trait};

#[extern_trait(crate = renamed_extern_trait, RenamedProxy)]
trait Renamed {
    fn new(v: i32) -> Self;
    fn value(&self) -> i32;
}

struct RenamedImpl(i32);

#[extern_trait(crate = renamed_extern_trait)]
impl Renamed for RenamedImpl {
    fn new(v: i32) -> Self {
        Self(v)
    }

    fn value(&self) -> i32 {
        self.0
    }
}

fn main() {
    let proxy = RenamedProxy::new(7);
    assert_eq!(proxy.value(), 7);
}

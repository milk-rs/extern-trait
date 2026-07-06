use extern_trait::{self as renamed_extern_trait, extern_trait};

#[extern_trait(crate = renamed_extern_trait, CratePathBeforeProxy)]
trait CratePathBeforeProxyApi {
    fn new(v: i32) -> Self;
    fn value(&self) -> i32;
}

struct CratePathBeforeProxyImpl(i32);

#[extern_trait(crate = renamed_extern_trait)]
impl CratePathBeforeProxyApi for CratePathBeforeProxyImpl {
    fn new(v: i32) -> Self {
        Self(v)
    }

    fn value(&self) -> i32 {
        self.0
    }
}

#[extern_trait(ProxyBeforeCratePath, crate = renamed_extern_trait)]
trait ProxyBeforeCratePathApi {
    fn new(v: i32) -> Self;
    fn value(&self) -> i32;
}

struct ProxyBeforeCratePathImpl(i32);

#[extern_trait(crate = renamed_extern_trait)]
impl ProxyBeforeCratePathApi for ProxyBeforeCratePathImpl {
    fn new(v: i32) -> Self {
        Self(v)
    }

    fn value(&self) -> i32 {
        self.0
    }
}

fn main() {
    let crate_first = CratePathBeforeProxy::new(7);
    assert_eq!(crate_first.value(), 7);

    let proxy_first = ProxyBeforeCratePath::new(11);
    assert_eq!(proxy_first.value(), 11);
}


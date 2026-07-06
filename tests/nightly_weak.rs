#![cfg(feature = "nightly-weak")]
#![feature(linkage)]

use ::extern_trait as renamed_extern_trait;
use extern_trait::extern_trait;

struct WeakDefaultCounter(u16);

#[extern_trait(default = WeakDefaultCounter, pub WeakCounterProxy)]
trait WeakCounter {
    fn new(value: u16) -> Self;
    fn value(&self) -> u16;
    fn route(&self) -> &'static str;
}

impl WeakCounter for WeakDefaultCounter {
    fn new(value: u16) -> Self {
        Self(value + 10)
    }

    fn value(&self) -> u16 {
        self.0
    }

    fn route(&self) -> &'static str {
        "default"
    }
}

struct RenamedDefaultCounter(u16);

#[renamed_extern_trait::extern_trait(
    default = RenamedDefaultCounter,
    crate = renamed_extern_trait,
    pub RenamedCounterProxy
)]
trait RenamedCounter {
    fn new(value: u16) -> Self;
    fn value(&self) -> u16;
    fn route(&self) -> &'static str;
}

impl RenamedCounter for RenamedDefaultCounter {
    fn new(value: u16) -> Self {
        Self(value + 100)
    }

    fn value(&self) -> u16 {
        self.0
    }

    fn route(&self) -> &'static str {
        "renamed-default"
    }
}

#[test]
fn proxy_dispatches_to_weak_default_when_no_strong_impl_is_linked() {
    let counter = WeakCounterProxy::new(32);

    assert_eq!(counter.value(), 42);
    assert_eq!(counter.route(), "default");
}

#[test]
fn renamed_crate_path_and_default_type_dispatch_to_weak_default() {
    let counter = RenamedCounterProxy::new(23);

    assert_eq!(counter.value(), 123);
    assert_eq!(counter.route(), "renamed-default");
}

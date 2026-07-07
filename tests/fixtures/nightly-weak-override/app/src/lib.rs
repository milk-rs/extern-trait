use api::{Counter, CounterProxy};
use extern_trait::extern_trait;

pub struct StrongCounter;

#[extern_trait(crate = extern_trait)]
impl Counter for StrongCounter {
    fn value() -> u16 {
        2
    }
}

#[test]
fn linked_downstream_impl_replaces_the_upstream_default() {
    assert_eq!(CounterProxy::value(), 2);
}

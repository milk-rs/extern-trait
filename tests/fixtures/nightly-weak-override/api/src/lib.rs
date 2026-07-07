#![feature(linkage)]

use extern_trait::extern_trait;

pub struct DefaultCounter;

#[extern_trait(default = DefaultCounter, pub CounterProxy)]
pub trait Counter {
    fn value() -> u16;
}

impl Counter for DefaultCounter {
    fn value() -> u16 {
        1
    }
}

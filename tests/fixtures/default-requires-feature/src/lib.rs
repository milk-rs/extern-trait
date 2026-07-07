use extern_trait::extern_trait;

struct DefaultCounter;

#[extern_trait(default = DefaultCounter, CounterProxy)]
trait Counter {
    fn new() -> Self;
}

impl Counter for DefaultCounter {
    fn new() -> Self {
        Self
    }
}

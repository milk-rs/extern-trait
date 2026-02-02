use extern_trait::extern_trait;

#[extern_trait(CounterProxy)]
trait Counter {
    fn new(value: u64) -> Self;
    fn get(&self) -> u64;
    fn increment(self) -> Self;
    fn add(self, other: Self) -> Self;
}

mod counter_impl {
    use super::*;

    #[derive(Clone, Copy)]
    struct CounterImpl(u64);

    #[extern_trait]
    impl Counter for CounterImpl {
        fn new(value: u64) -> Self {
            Self(value)
        }

        fn get(&self) -> u64 {
            self.0
        }

        fn increment(self) -> Self {
            Self(self.0 + 1)
        }

        fn add(self, other: Self) -> Self {
            Self(self.0 + other.0)
        }
    }
}

#[test]
fn test_by_value_self() {
    let counter = CounterProxy::new(10);
    assert_eq!(counter.get(), 10);

    // Test by-value self with Self return
    let counter = counter.increment();
    assert_eq!(counter.get(), 11);

    let counter = counter.increment().increment();
    assert_eq!(counter.get(), 13);
}

#[test]
fn test_by_value_self_with_other() {
    let a = CounterProxy::new(5);
    let b = CounterProxy::new(7);

    // Test by-value self AND by-value Self parameter
    let result = a.add(b);
    assert_eq!(result.get(), 12);
}

#[test]
fn test_chained_operations() {
    let result = CounterProxy::new(1)
        .increment()
        .add(CounterProxy::new(10))
        .increment()
        .increment();
    assert_eq!(result.get(), 14);
}

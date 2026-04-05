use extern_trait::extern_trait;

#[extern_trait(CalcProxy)]
trait Calc {
    fn new(v: i64) -> Self;
    fn get(&self) -> i64;
    fn set(&mut self, v: i64);
    fn add(self, rhs: Self) -> Self;
    fn scale(self, rhs: i64) -> Self;
    fn add_values(a: i64, b: i64) -> i64;
}

mod calc_impl {
    use super::*;

    #[derive(Clone, Copy)]
    struct CalcImpl(i64);

    #[extern_trait]
    impl Calc for CalcImpl {
        fn new(v: i64) -> Self {
            Self(v)
        }

        fn get(&self) -> i64 {
            self.0
        }

        fn set(&mut self, v: i64) {
            self.0 = v;
        }

        fn add(self, rhs: Self) -> Self {
            Self(self.0 + rhs.0)
        }

        fn scale(self, rhs: i64) -> Self {
            Self(self.0 * rhs)
        }

        fn add_values(a: i64, b: i64) -> i64 {
            a + b
        }
    }
}

#[test]
fn proxy_dispatches_instance_and_static_methods() {
    let mut v = CalcProxy::new(10);
    assert_eq!(v.get(), 10);

    v.set(15);
    assert_eq!(v.get(), 15);
    assert_eq!(CalcProxy::add_values(40, 2), 42);
}

#[test]
fn proxy_keeps_by_value_self_flow_composable() {
    let out = CalcProxy::new(2)
        .add(CalcProxy::new(3))
        .scale(4)
        .add(CalcProxy::new(1));

    assert_eq!(out.get(), 21);
}

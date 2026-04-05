use std::sync::atomic::{AtomicUsize, Ordering};

use extern_trait::extern_trait;

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[extern_trait(DropProxy)]
trait DropApi {
    fn new(v: usize) -> Self;
    fn value(&self) -> usize;
}

mod drop_impl {
    use super::*;

    struct DropImpl(usize);

    impl Drop for DropImpl {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[extern_trait]
    impl DropApi for DropImpl {
        fn new(v: usize) -> Self {
            Self(v)
        }

        fn value(&self) -> usize {
            self.0
        }
    }
}

#[test]
fn proxy_drop_runs_impl_drop_exactly_once() {
    DROPS.store(0, Ordering::SeqCst);

    {
        let p = DropProxy::new(7);
        assert_eq!(p.value(), 7);
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }

    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

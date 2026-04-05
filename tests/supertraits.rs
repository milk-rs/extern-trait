use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Debug, Display};
use std::panic::{RefUnwindSafe, UnwindSafe};

use extern_trait::extern_trait;

#[extern_trait(ScoreProxy)]
trait ScoreApi:
    Send
    + Sync
    + Unpin
    + UnwindSafe
    + RefUnwindSafe
    + Clone
    + Debug
    + Display
    + Default
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + AsRef<u64>
    + AsMut<u64>
    + Borrow<u64>
    + BorrowMut<u64>
{
    fn new(value: u64) -> Self;
    fn value(&self) -> u64;
}

mod score_impl {
    use super::*;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
    struct Score(u64);

    impl Display for Score {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl AsRef<u64> for Score {
        fn as_ref(&self) -> &u64 {
            &self.0
        }
    }

    impl AsMut<u64> for Score {
        fn as_mut(&mut self) -> &mut u64 {
            &mut self.0
        }
    }

    impl Borrow<u64> for Score {
        fn borrow(&self) -> &u64 {
            &self.0
        }
    }

    impl BorrowMut<u64> for Score {
        fn borrow_mut(&mut self) -> &mut u64 {
            &mut self.0
        }
    }

    #[extern_trait]
    impl ScoreApi for Score {
        fn new(value: u64) -> Self {
            Self(value)
        }

        fn value(&self) -> u64 {
            self.0
        }
    }
}

#[test]
fn proxy_forwards_marker_supertraits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_unpin<T: Unpin>() {}
    fn assert_unwind_safe<T: UnwindSafe>() {}
    fn assert_ref_unwind_safe<T: RefUnwindSafe>() {}

    assert_send::<ScoreProxy>();
    assert_sync::<ScoreProxy>();
    assert_unpin::<ScoreProxy>();
    assert_unwind_safe::<ScoreProxy>();
    assert_ref_unwind_safe::<ScoreProxy>();
}

#[test]
fn proxy_forwards_standard_supertraits() {
    let mut a = ScoreProxy::new(10);
    let b = ScoreProxy::new(20);

    assert_eq!(a.value(), 10);
    assert_eq!(a, ScoreProxy::new(10));
    assert!(a < b);
    assert_eq!(a.cmp(&b), core::cmp::Ordering::Less);
    assert_eq!(a.partial_cmp(&b), Some(core::cmp::Ordering::Less));
    assert_eq!(format!("{}", a), "10");
    assert_eq!(format!("{:?}", a), "Score(10)");

    *a.as_mut() = 11;
    assert_eq!(*a.as_ref(), 11);

    *<ScoreProxy as BorrowMut<u64>>::borrow_mut(&mut a) = 12;
    assert_eq!(*<ScoreProxy as Borrow<u64>>::borrow(&a), 12);

    let c = a.clone();
    let d = c.clone();
    assert_eq!(d.value(), 12);

    let default = ScoreProxy::default();
    assert_eq!(default.value(), 0);
}

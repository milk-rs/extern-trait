use std::fmt::Debug;

use extern_trait::extern_trait;

#[extern_trait(ResourceProxy)]
trait Resource: Send + Sync + AsRef<str> + Debug + Default + Clone {
    fn new() -> Self;
    fn count(&self) -> usize;
}

mod resource_impl {
    use std::sync::{Arc, LazyLock};

    use super::*;

    static GLOBAL: LazyLock<Arc<String>> = LazyLock::new(|| Arc::new("Hello, world!".to_string()));

    #[derive(Debug, Clone)]
    struct ActualResource(Arc<String>);

    impl AsRef<str> for ActualResource {
        fn as_ref(&self) -> &str {
            self.0.as_ref()
        }
    }

    unsafe impl extern_trait::IntRegRepr for ActualResource {}

    #[extern_trait]
    impl Resource for ActualResource {
        fn new() -> Self {
            Self(GLOBAL.clone())
        }

        fn count(&self) -> usize {
            Arc::strong_count(&self.0)
        }
    }

    impl Default for ActualResource {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[test]
fn test_resource() {
    struct _AssertSend
    where
        ResourceProxy: Send;
    struct _AssertSync
    where
        ResourceProxy: Sync;

    let res = ResourceProxy::new();
    assert_eq!(res.count(), 2);
    assert_eq!(res.as_ref(), "Hello, world!");

    {
        let res = ResourceProxy::default();
        assert_eq!(res.count(), 3);
    }

    assert_eq!(res.count(), 2);

    assert_eq!(
        format!("{:?}", res.clone()),
        "ActualResource(\"Hello, world!\")"
    );
}

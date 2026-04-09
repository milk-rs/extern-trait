use extern_trait::extern_trait;

/// # Safety
/// This trait is safe.
#[extern_trait(UnsafeProxy)]
unsafe trait UnsafeTrait {}

struct UnsafeImpl;

#[extern_trait]
impl UnsafeTrait for UnsafeImpl {}

fn main() {}

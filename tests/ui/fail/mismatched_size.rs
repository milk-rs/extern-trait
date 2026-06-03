use extern_trait::extern_trait;

#[extern_trait(ptr_count = 3, MismatchedSizeProxy)]
trait MismatchedSizeApi {
    fn new() -> Self;
}

struct MismatchedSize([usize; 2]);
#[extern_trait(ptr_count = 2)]
impl MismatchedSizeApi for MismatchedSize {
    fn new() -> Self {
        Self([0; 3])
    }
}

fn main() {
    MismatchedSizeProxy::new();
}


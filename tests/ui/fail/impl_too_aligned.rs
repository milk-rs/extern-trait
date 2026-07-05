use extern_trait::extern_trait;

#[extern_trait(Proxy)]
trait Api {
    fn new() -> Self;
}

#[cfg_attr(target_pointer_width = "32", repr(C, align(8)))]
#[cfg_attr(target_pointer_width = "64", repr(C, align(16)))]
struct TooAligned(usize, usize);

#[extern_trait]
impl Api for TooAligned {
    fn new() -> Self {
        Self(0, 0)
    }
}

fn main() {}

use extern_trait::extern_trait;

#[extern_trait(Proxy)]
trait Api {
    fn new() -> Self;
}

#[repr(C, align(16))]
struct TooAligned(usize);

#[extern_trait]
impl Api for TooAligned {
    fn new() -> Self {
        Self(0)
    }
}

fn main() {}

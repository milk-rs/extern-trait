use extern_trait::extern_trait;

#[extern_trait(Proxy)]
trait Api {
    fn new() -> Self;
}

#[repr(align(16))]
struct TooAligned;

#[extern_trait]
impl Api for TooAligned {
    fn new() -> Self {
        Self
    }
}

fn main() {}

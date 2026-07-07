use extern_trait::extern_trait;

#[extern_trait(Proxy)]
trait Api {
    fn new() -> Self;
}

struct Impl;

#[extern_trait(default = Impl)]
impl Api for Impl {
    fn new() -> Self {
        Self
    }
}

fn main() {}

use extern_trait::extern_trait;

#[extern_trait(Proxy)]
trait Api {
    fn new() -> Self;
}

struct Impl<T>(T);

#[extern_trait]
impl<T> Api for Impl<T> {
    fn new() -> Self {
        panic!()
    }
}

fn main() {}

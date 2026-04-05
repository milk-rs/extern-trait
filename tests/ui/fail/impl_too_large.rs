use extern_trait::extern_trait;

#[extern_trait(Proxy)]
trait Api {
    fn new() -> Self;
}

struct TooLarge([usize; 3]);

#[extern_trait]
impl Api for TooLarge {
    fn new() -> Self {
        Self([0; 3])
    }
}

fn main() {}

use extern_trait::extern_trait;

struct X;

#[extern_trait]
impl X {
    fn new() -> Self {
        Self
    }
}

fn main() {}

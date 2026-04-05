use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    type Output;
    fn new() -> Self;
}

fn main() {}

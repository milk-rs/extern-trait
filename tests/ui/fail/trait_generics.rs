use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad<T> {
    fn new(v: T) -> Self;
}

fn main() {}

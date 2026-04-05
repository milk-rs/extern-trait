use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    fn value<T>(&self, t: T) -> T;
}

fn main() {}

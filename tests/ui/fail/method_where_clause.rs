use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    fn value(&self) -> i32
    where
        Self: Sized;
}

fn main() {}

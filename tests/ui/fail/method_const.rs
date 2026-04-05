use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    const fn value(&self) -> i32;
}

fn main() {}

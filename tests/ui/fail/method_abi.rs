use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    extern "C" fn value(&self) -> i32;
}

fn main() {}

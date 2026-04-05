use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    async fn value(&self) -> i32;
}

fn main() {}

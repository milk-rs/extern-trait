use extern_trait::extern_trait;

#[extern_trait(BadProxy)]
trait Bad {
    fn nested(self: Box<Self>) -> Self;
}

fn main() {}

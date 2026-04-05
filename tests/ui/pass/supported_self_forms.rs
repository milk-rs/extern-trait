use extern_trait::extern_trait;

#[extern_trait(ShapeProxy)]
trait Shape {
    fn new(v: i32) -> Self;
    fn by_ref(&self) -> i32;
    fn by_mut_ref(&mut self, delta: i32);
    fn consume(self) -> Self;
}

struct ShapeImpl(i32);

#[extern_trait]
impl Shape for ShapeImpl {
    fn new(v: i32) -> Self {
        Self(v)
    }

    fn by_ref(&self) -> i32 {
        self.0
    }

    fn by_mut_ref(&mut self, delta: i32) {
        self.0 += delta;
    }

    fn consume(self) -> Self {
        Self(self.0 + 1)
    }
}

fn main() {
    let mut v = ShapeProxy::new(1);
    assert_eq!(v.by_ref(), 1);
    v.by_mut_ref(2);
    assert_eq!(v.by_ref(), 3);
    v.by_mut_ref(7);
    assert_eq!(v.consume().by_ref(), 11);
}

use extern_trait::extern_trait;

#[extern_trait(ptr_count = 0, MarkerProxy)]
pub trait UnsizedMarker: Clone + Copy {
    fn new() -> Self;
    fn get(self) -> usize;
}

#[derive(Copy, Clone)]
pub struct ZeroSize;
#[extern_trait(ptr_count = 0)]
impl UnsizedMarker for ZeroSize {
    fn new() -> Self {
        ZeroSize
    }
    fn get(self) -> usize {
        3
    }
}

#[extern_trait(unsized, UnsizedProxy)]
pub trait UnsizedTrait {
    fn is_happy(&self) -> bool;
}

pub struct Huge([u8; 4096]);
#[extern_trait(unsized)]
impl UnsizedTrait for Huge {
    fn is_happy(&self) -> bool {
        self.0[28] > 3
    }
}

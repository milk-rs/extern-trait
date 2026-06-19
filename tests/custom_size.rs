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

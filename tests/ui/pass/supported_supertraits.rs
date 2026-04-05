use core::fmt::{Debug, Display};

use extern_trait::extern_trait;

#[extern_trait(ScoreProxy)]
trait ScoreApi:
    Send + Sync + Clone + Debug + Display + Default + PartialEq + Eq + PartialOrd + Ord
{
    fn new(v: i32) -> Self;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Score(i32);

impl Display for Score {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[extern_trait]
impl ScoreApi for Score {
    fn new(v: i32) -> Self {
        Self(v)
    }
}

fn main() {
    let a = ScoreProxy::new(1);
    let b = ScoreProxy::default();
    let _ = a.clone();
    let _ = format!("{} {:?}", a, b);
    let _ = a == b;
    let _ = a.cmp(&b);
}

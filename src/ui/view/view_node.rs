use super::*;

pub trait ViewNode: View + Node {
    fn rating(&self, context: &Context) -> i32 {
        0
    }
}

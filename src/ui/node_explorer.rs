use std::marker::PhantomData;

use super::*;

pub struct NodeExplorer<T: NodeViewFns> {
    pd: PhantomData<T>,
}

impl<T: NodeViewFns> NodeExplorer<T> {
    pub fn new() -> NodeExplorer<T> {
        Self { pd: PhantomData }
    }
    pub fn ui(&mut self, context: &mut Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let vctx = ViewContextNew::new(ui);
        for n in context.world_mut()?.query::<&T>().iter(context.world()?) {
            n.view_node(vctx, context, ui);
        }
        Ok(())
    }
}

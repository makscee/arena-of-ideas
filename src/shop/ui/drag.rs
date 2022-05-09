use super::*;

pub struct DragWidget {
    child: Box<dyn Widget>,
}

impl DragWidget {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {child}
    }
}

impl Widget for DragWidget {
    fn calc_constraints(&mut self, cx: &ConstraintsContext) -> Constraints {
        Constraints::default()
    }
    fn layout_children(&mut self, cx: &mut LayoutContext) {
        
    }
}

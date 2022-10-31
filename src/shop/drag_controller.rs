use super::*;

pub struct DragController<T>
where
    T: DragTarget,
{
    pub drag_target: Option<T>,
    pub source: DragSource,
}

impl<T: DragTarget> DragController<T> {
    pub fn new() -> Self {
        Self {
            drag_target: None,
            source: DragSource::Team,
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum DragSource {
    Team,
    Shop,
}

pub trait Touchable {
    fn is_touched(&self, pointer_position: Vec2<f32>) -> bool {
        let is_touched = self.touch_box().contains(pointer_position);
        is_touched
    }

    fn touch_box(&self) -> AABB<f32>;
}

pub trait DragTarget: Touchable {
    fn drag(&mut self, pointer_position: Vec2<R32>);
    fn restore(&mut self);
    fn position(&self) -> Vec2<f32>;
}

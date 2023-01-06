use super::*;

pub struct Game {
    pub geng: Geng,
    pub logic: Logic,
    pub model: Model,
    pub assets: Rc<Assets>,
    pub view: View,
    pub state: StateManager,
}

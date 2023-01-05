use super::*;

pub struct Game {
    pub logic: Logic,
    pub model: Model,
    pub assets: Assets,
    pub view: View,
    pub state: StateManager,
}

use super::*;

use crate::StateManager;
pub struct Game {
    pub geng: Geng,
    pub logic: Logic,
    pub assets: Rc<Assets>,
    pub view: View,
    pub state_manager: StateManager,
    pub schedule: Schedule,
    pub world: World,
    pub resources: Resources,
}

use ron::to_string;

use super::*;

pub struct ReportPlugin;

impl ReportPlugin {
    pub fn save_to_clipboard(world: &mut World) {
        let battle = match world.resource::<State<GameState>>().get() {
            GameState::Battle => world.resource::<BattleData>().clone(),
            GameState::Shop => {
                ShopPlugin::load_next_battle(world);
                world.resource::<BattleData>().clone()
            }
            _ => {
                return;
            }
        };
        save_to_clipboard(&to_string(&battle).unwrap(), world);
    }
}

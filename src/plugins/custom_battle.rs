use super::*;

pub struct CustomBattlePlugin;

impl Plugin for CustomBattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CustomBattle), Self::on_enter);
    }
}

impl CustomBattlePlugin {
    fn on_enter(world: &mut World) {
        Self::load_teams(world);
        GameState::change(GameState::Battle, world);
    }

    fn load_teams(world: &mut World) {
        let cb = Options::get_custom_battle(world).clone();
        BattlePlugin::load_teams(cb.left, cb.right, None, world);
    }
}

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "6cb61798-ec2c-4875-bec8-464c4f56c229"]
pub struct CustomBattleData {
    pub left: PackedTeam,
    pub right: PackedTeam,
}

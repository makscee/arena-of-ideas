use super::*;

pub struct CustomBattlePlugin;

impl Plugin for CustomBattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LastBattle), Self::on_enter_last)
            .add_systems(OnEnter(GameState::CustomBattle), Self::on_enter);
    }
}

impl CustomBattlePlugin {
    fn on_enter(world: &mut World) {
        Pools::get_mut(world).only_local_cache = true;
        Self::load_teams(world);
        GameState::change(GameState::Battle, world);
    }
    fn on_enter_last(world: &mut World) {
        Pools::get_mut(world).only_local_cache = true;
        Self::load_teams_last(world);
        GameState::change(GameState::Battle, world);
    }

    fn load_teams(world: &mut World) {
        let cb = Options::get_custom_battle(world).clone();
        BattlePlugin::load_teams(cb.left, cb.right, None, world);
    }
    fn load_teams_last(world: &mut World) {
        let mut data = PersistentData::load(world).last_battle;
        data.run_id = None;
        world.insert_resource(data);
    }
}

#[derive(Asset, Deserialize, TypePath, Debug, Clone)]
pub struct CustomBattleData {
    pub left: PackedTeam,
    pub right: PackedTeam,
}

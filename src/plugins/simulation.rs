use super::*;

pub struct SimulationPlugin;

impl SimulationPlugin {
    pub fn run(left: PackedTeam, right: PackedTeam, world: &mut World) -> Result<BattleResult> {
        SkipVisual::set_active(true, world);
        BattlePlugin::load_teams(left, right, world);
        let result = BattlePlugin::run_battle(world);
        SkipVisual::set_active(false, world);
        result
    }

    pub fn clear(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
        GameTimer::get().reset();
    }
}

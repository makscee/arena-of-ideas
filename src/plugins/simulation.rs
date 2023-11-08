use super::*;

pub struct SimulationPlugin;

impl SimulationPlugin {
    pub fn run(left: PackedTeam, right: PackedTeam, world: &mut World) -> BattleResult {
        left.unpack(Faction::Left, world);
        right.unpack(Faction::Right, world);
        let result = BattlePlugin::run_battle(world);
        UnitPlugin::despawn_all(world);
        GameTimer::get_mut(world).reset();
        result
    }
}

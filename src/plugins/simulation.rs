use super::*;

pub struct SimulationPlugin;

impl SimulationPlugin {
    pub fn run(left: PackedTeam, right: PackedTeam, world: &mut World) -> BattleResult {
        left.unpack(Faction::Left, world);
        right.unpack(Faction::Right, world);
        let result = BattlePlugin::run_battle(world);
        UnitPlugin::clear_world(world);
        result
    }
}

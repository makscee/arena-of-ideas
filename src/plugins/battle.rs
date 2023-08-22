use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::setup);
    }
}

impl BattlePlugin {
    pub fn setup(world: &mut World) {
        let bs = Options::get_custom_battle(world).clone();
        bs.unpack(world);
        Self::translate_to_slots(world);
    }

    fn translate_to_slots(world: &mut World) {
        for (unit, faction) in
            UnitPlugin::collect_factions(&HashSet::from([Faction::Left, Faction::Right]), world)
        {
            let slot = VarState::get_value_from_world(unit, VarName::Slot, world)
                .unwrap()
                .get_int()
                .unwrap() as usize;
            UnitPlugin::translate_unit(unit, UnitPlugin::get_slot_position(faction, slot), world)
        }
    }
}

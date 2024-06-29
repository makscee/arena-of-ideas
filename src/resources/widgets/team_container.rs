use super::*;

pub struct TeamContainer {
    center: Pos2,
    offset: egui::Vec2,
    faction: Faction,
}

impl TeamContainer {
    pub fn new(center: Pos2, faction: Faction, offset: egui::Vec2) -> Self {
        Self {
            center,
            faction,
            offset,
        }
    }
    pub fn ui(self, ui: &mut Ui, world: &mut World) {
        let units = UnitPlugin::collect_faction(self.faction, world);
        let start_pos = self.center - (units.len() as f32 * 0.5 + 0.5) * self.offset;
        for entity in units {
            let state = VarState::get(entity, world);
            let pos = start_pos + state.get_int(VarName::Slot).unwrap() as f32 * self.offset;
            let pos = screen_to_world(pos.to_bvec2(), world);
            VarState::get_mut(entity, world).set_vec2(VarName::Position, pos);
        }
    }
}

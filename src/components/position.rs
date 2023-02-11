use super::*;

pub struct Position(pub vec2<f32>);

impl VarsProvider for Position {
    fn extend_vars(&self, vars: &mut Vars) {
        vars.insert(VarName::Position, Var::Vec2(self.0));
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(vec2::ZERO)
    }
}

impl Position {
    pub fn update_from_slot_faction(&mut self, slot: usize, faction: &Faction) {
        self.0 = Self::from_slot_faction(slot, faction);
    }

    pub fn from_slot_faction(slot: usize, faction: &Faction) -> vec2<f32> {
        let faction_mul: vec2<f32> = vec2(
            match faction {
                Faction::Light => -1.0,
                Faction::Dark => 1.0,
                Faction::Team => -1.0,
                Faction::Shop => 1.0,
            },
            1.0,
        );
        return match slot == 1 {
            true => vec2(1.5, 0.0),
            false => vec2((slot as f32 - 1.0) * 2.5, -4.0),
        } * faction_mul;
    }
}

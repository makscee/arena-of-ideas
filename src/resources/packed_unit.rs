use super::*;

#[derive(Asset, Deserialize, Serialize, TypePath, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    #[serde(default = "default_empty")]
    pub name: String,
    #[serde(default = "default_one")]
    pub pwr: i32,
    #[serde(default = "default_one")]
    pub hp: i32,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub statuses: Vec<(String, i32)>,
}

fn default_one() -> i32 {
    1
}

fn default_empty() -> String {
    "_empty".to_owned()
}

impl PackedUnit {
    pub fn unpack(mut self, parent: Entity, slot: Option<i32>, world: &mut World) -> Entity {
        let entity = world.spawn_empty().set_parent(parent).insert(Unit).id();
        debug!("unpack unit: {entity:?} {self:?}");
        self.state = self.generate_state(world);
        self.state
            .init(VarName::Slot, VarValue::Int(slot.unwrap_or_default()));
        Status {
            name: "_local".to_owned(),
            trigger: self.trigger,
        }
        .spawn(entity, world);
        self.state.add_status(
            "_local".to_owned(),
            VarState::default()
                .init(VarName::Charges, VarValue::Int(1))
                .take(),
        );
        self.state.attach(entity, world);
        for (status, charges) in self.statuses {
            let _ = Status::change_charges(&status, entity, charges, world);
        }
        Status::refresh_mappings(entity, world);
        entity
    }

    pub fn generate_state(&self, world: &World) -> VarState {
        let mut state = self.state.clone();
        state
            .init(VarName::Hp, VarValue::Int(self.hp))
            .init(VarName::Pwr, VarValue::Int(self.pwr))
            .init(VarName::Name, VarValue::String(self.name.clone()))
            .init(VarName::Position, VarValue::Vec2(default()));
        if !state.has_value(VarName::Dmg) {
            state.init(VarName::Dmg, VarValue::Int(0));
        }
        state
    }
}

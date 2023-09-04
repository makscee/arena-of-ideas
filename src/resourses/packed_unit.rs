use super::*;

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
pub struct PackedUnit {
    pub hp: i32,
    pub atk: i32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub representation: Representation,
    pub state: VarState,
}

impl PackedUnit {
    pub fn unpack(mut self, faction: Faction, slot: Option<usize>, world: &mut World) {
        debug!("Unpack unit {:?}", &self);
        let entity = Options::get_unit_rep(world).clone().unpack(None, world);
        world
            .entity_mut(entity)
            .insert(PickableBundle::default())
            .insert(RaycastPickTarget::default())
            .insert(On::<Pointer<Over>>::run(UnitPlugin::hover_unit))
            .insert(On::<Pointer<Out>>::run(UnitPlugin::unhover_unit));
        {
            let entity = self.representation.unpack(Some(entity), world);
            world.entity_mut(entity).insert(UnitRepresentation);
        }
        self.state
            .insert(VarName::Hp, VarValue::Int(self.hp))
            .insert(VarName::Atk, VarValue::Int(self.atk))
            .insert(VarName::Name, VarValue::String(self.name.clone()))
            .insert(VarName::Position, VarValue::Vec2(default()))
            .insert(
                VarName::Slot,
                VarValue::Int(slot.unwrap_or_default() as i32),
            )
            .insert(VarName::Faction, VarValue::Faction(faction))
            .insert(
                VarName::Description,
                VarValue::String(self.description.to_owned()),
            );
        world
            .entity_mut(entity)
            .insert(Unit)
            .insert(Name::new(self.name))
            .insert(self.state);

        Options::get_statuses(world)
            .get("Test")
            .unwrap()
            .clone()
            .unpack(entity, world)
            .unwrap();
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let rep_entity = *world
            .get::<Children>(entity)
            .unwrap()
            .into_iter()
            .find(|x| world.get::<UnitRepresentation>(**x).is_some())
            .unwrap();
        let representation = Representation::pack(rep_entity, world);
        let state = VarState::get(entity, world).clone();
        let hp = state.get_int(VarName::Hp).unwrap();
        let atk = state.get_int(VarName::Atk).unwrap();
        let name = state.get_string(VarName::Name).unwrap();
        let description = state.get_string(VarName::Description).unwrap();
        Self {
            hp,
            atk,
            name,
            representation,
            state,
            description,
        }
    }
}

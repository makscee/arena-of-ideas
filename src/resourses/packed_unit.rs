use super::*;

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
pub struct PackedUnit {
    pub hp: i32,
    pub atk: i32,
    pub name: String,
    pub representation: Representation,
    pub state: VarState,
}

impl PackedUnit {
    pub fn unpack(mut self, world: &mut World) {
        let entity = Options::get_unit_rep(world).clone().unpack(None, world);
        world
            .entity_mut(entity)
            .insert(PickableBundle::default())
            .insert(RaycastPickTarget::default())
            .insert(On::<Pointer<Over>>::run(hover_unit));
        self.representation.unpack(Some(entity), world);
        self.state
            .insert(VarName::Hp, VarValue::Int(self.hp))
            .insert(VarName::Atk, VarValue::Int(self.atk))
            .insert(VarName::Name, VarValue::String(self.name))
            .insert(VarName::Position, VarValue::Vec2(default()));
        world.entity_mut(entity).insert(Unit).insert(self.state);
    }
}

fn hover_unit(event: Listener<Pointer<Over>>) {
    debug!("Hover over unit start {:?}", event.target);
}

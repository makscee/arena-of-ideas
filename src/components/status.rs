use super::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PackedStatus {
    pub name: String,
    // pub trigger: Trigger,
    pub representation: Option<Representation>,
    #[serde(default)]
    pub state: VarState,
}

#[derive(Component)]
pub struct Status {
    pub name: String,
}

impl PackedStatus {
    pub fn unpack(mut self, entity: Entity, world: &mut World) -> Result<()> {
        let entity = self.representation.unwrap().unpack(Some(entity), world);
        if self.state.get_int(VarName::Charges).is_err() {
            self.state.insert(VarName::Charges, VarValue::Int(1));
        }
        world
            .get_entity_mut(entity)
            .context("Entity doesn't exist")?
            .insert((
                Status {
                    name: self.name.clone(),
                },
                Name::from(self.name),
                self.state,
            ));
        Ok(())
    }
}

impl Status {
    pub fn change_charges(
        status_name: &str,
        entity: Entity,
        delta: i32,
        world: &mut World,
    ) -> Result<()> {
        let children = world
            .get_entity(entity)
            .context("Unit not found")?
            .get::<Children>()
            .unwrap()
            .to_vec();
        for child in children {
            if let Some(status) = world.entity_mut(child).get_mut::<Status>() {
                if status.name.eq(status_name) {
                    VarState::change_int(child, VarName::Charges, delta, world)?;
                }
            }
        }
        Ok(())
    }
}

use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackedStatus {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub polarity: i32,
    #[serde(default)]
    pub state: VarState,
    pub trigger: Trigger,
}

#[derive(Component, Clone)]
pub struct Status {
    pub name: String,
    pub trigger: Trigger,
}

impl PackedStatus {
    fn unpack(mut self, owner: Entity, world: &mut World) -> Entity {
        if self.state.get_int(VarName::Charges).is_err() {
            self.state.init(VarName::Charges, VarValue::Int(0));
        }
        self.state
            .init(
                VarName::Description,
                VarValue::String(self.description.to_owned()),
            )
            .init(VarName::Name, VarValue::String(self.name.to_owned()))
            .init(VarName::Polarity, VarValue::Int(self.polarity));
        let entity = Status::spawn(owner, &self, world);
        VarState::get_mut(owner, world).add_status(self.name, self.state);
        entity
    }
}

impl Status {
    fn spawn(owner: Entity, status: &PackedStatus, world: &mut World) -> Entity {
        world
            .spawn((
                Name::new(status.name.clone()),
                Status {
                    name: status.name.clone(),
                    trigger: status.trigger.clone(),
                },
            ))
            .set_parent(owner)
            .id()
    }
    pub fn change_charges(
        status: &str,
        entity: Entity,
        delta: i32,
        world: &mut World,
    ) -> Result<i32> {
        let mut owner_state = VarState::try_get_mut(entity, world)?;
        let state = if let Some(state) = owner_state.get_status_mut(status) {
            state
        } else {
            PackedStatus {
                name: todo!(),
                description: todo!(),
                polarity: todo!(),
                state: todo!(),
                trigger: todo!(),
            }
            .unpack(entity, world);
            VarState::get_mut(entity, world)
                .get_status_mut(status)
                .unwrap()
        };
        Ok(state.change_int(VarName::Charges, delta))
    }
}

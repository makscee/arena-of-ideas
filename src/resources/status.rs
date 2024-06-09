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
    pub fn change_charges(status: &str, entity: Entity, delta: i32, world: &mut World) -> i32 {
        if let Some(state) = VarState::get_mut(entity, world).get_status_mut(status) {
            state.change_int(VarName::Charges, delta)
        } else {
            GameAssets::get(world)
                .statuses
                .get(status)
                .unwrap()
                .clone()
                .unpack(entity, world);
            Self::change_charges(status, entity, delta, world)
        }
    }
    pub fn get_charges(status: &str, entity: Entity, world: &World) -> Result<i32> {
        Ok(VarState::try_get(entity, world)?
            .get_status(status)
            .and_then(|s| s.get_int(VarName::Charges).ok())
            .unwrap_or_default())
    }
    pub fn collect_statuses(entity: Entity, world: &World) -> Vec<Status> {
        get_children(entity, world)
            .into_iter()
            .filter_map(|e| world.get::<Status>(e).cloned())
            .collect_vec()
    }
    pub fn refresh_mappings(entity: Entity, world: &mut World) {
        let statuses = Self::collect_statuses(entity, world);
        for Status { name, trigger } in statuses {
            let context = &Context::new(entity).set_status(entity, name.clone()).take();
            let mappings = trigger.collect_mappings(context, world);
            let mut state = VarState::get_mut(entity, world);
            for (var, value) in mappings {
                if !state
                    .get_key_value_last(&name, var)
                    .unwrap_or_default()
                    .eq(&value)
                {
                    state.set_key_value(name.clone(), var, value);
                }
            }
        }
    }
}

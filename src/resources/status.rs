use ron::to_string;

use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackedStatus {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub polarity: i8,
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
            .init(VarName::Polarity, VarValue::Int(self.polarity as i32));
        let entity = Status {
            name: self.name.clone(),
            trigger: self.trigger.clone(),
        }
        .spawn(owner, world);
        VarState::get_mut(owner, world).add_status(self.name, self.state);
        entity
    }
}

impl Status {
    pub fn spawn(self, owner: Entity, world: &mut World) -> Entity {
        world
            .spawn((Name::new(self.name.clone()), self))
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
    pub fn collect_statuses(entity: Entity, world: &World) -> Vec<(Entity, Status)> {
        get_children(entity, world)
            .into_iter()
            .filter_map(|e| world.get::<Status>(e).cloned().map(|s| (e, s)))
            .collect_vec()
    }
    pub fn collect_active_statuses(entity: Entity, world: &World) -> Vec<(Entity, Status)> {
        Self::collect_statuses(entity, world)
            .into_iter()
            .filter(|(_, s)| {
                VarState::get(entity, world)
                    .get_status(&s.name)
                    .is_some_and(|s| s.get_int(VarName::Charges).unwrap_or_default() > 0)
            })
            .collect_vec()
    }
    pub fn notify(event: &Event, context: &Context, world: &mut World) -> bool {
        let owner = context.owner();
        let mut result = false;
        for (status, Status { name, mut trigger }) in Self::collect_active_statuses(owner, world) {
            if context.has_status(status) {
                continue;
            }
            let context = context.clone().set_status(status, name).take();
            result |= trigger.fire(event, &context, world);
            world.get_mut::<Status>(status).unwrap().trigger = trigger;
        }
        result
    }
    pub fn refresh_mappings(owner: Entity, world: &mut World) {
        let statuses = Self::collect_statuses(owner, world);
        for (status, Status { name, trigger }) in statuses {
            let context = &Context::new(owner).set_status(status, name.clone()).take();
            let mappings = trigger.collect_mappings(context, world);
            let mut state = VarState::get_mut(owner, world);
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

impl From<TStatus> for PackedStatus {
    fn from(value: TStatus) -> Self {
        Self {
            name: value.name,
            description: value.description,
            polarity: value.polarity,
            state: default(),
            trigger: ron::from_str::<Trigger>(&value.trigger).unwrap(),
        }
    }
}

impl From<PackedStatus> for TStatus {
    fn from(value: PackedStatus) -> Self {
        Self {
            name: value.name,
            description: value.description,
            polarity: value.polarity,
            trigger: to_string(&value.trigger).unwrap(),
        }
    }
}

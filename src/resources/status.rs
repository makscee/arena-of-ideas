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
        if self.state.get_value_last(VarName::Charges).is_err() {
            self.state.init(VarName::Charges, VarValue::Int(0));
        }
        self.state
            .init(
                VarName::Description,
                VarValue::String(self.description.to_owned()),
            )
            .init(VarName::Name, VarValue::String(self.name.to_owned()))
            .init(VarName::Color, name_color(&self.name).into())
            .init(VarName::Polarity, VarValue::Int(self.polarity as i32));
        let entity = Status {
            name: self.name.clone(),
            trigger: self.trigger.clone(),
        }
        .spawn(owner, world);
        debug!("unpack status {} {entity:?}", self.name);
        RepresentationPlugin::get_by_id(self.name.clone())
            .unwrap_or(GameAssets::get(world).status_rep.clone())
            .unpack(entity, world);
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
    pub fn change_charges_with_text(
        name: &str,
        entity: Entity,
        delta: i32,
        world: &mut World,
    ) -> i32 {
        let charges = Self::change_charges(name, entity, delta, world);
        let delta_str = if delta >= 0 {
            format!("+{delta}")
        } else {
            delta.to_string()
        };
        TextColumnPlugin::add(
            entity,
            "status "
                .cstr()
                .push(name.cstr_cs(name_color(name), CstrStyle::Bold))
                .push(format!(" {delta_str} ({charges})").cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold))
                .take(),
            world,
        );
        charges
    }
    pub fn change_charges(name: &str, entity: Entity, delta: i32, world: &mut World) -> i32 {
        if let Some(state) = VarState::get_mut(entity, world).get_status_mut(name) {
            let visible = state
                .get_value_last(VarName::Visible)
                .and_then(|v| v.get_bool())
                .unwrap_or(true);
            let charges = state.change_int(VarName::Charges, delta);
            if visible != (charges > 0) {
                state.set_value(VarName::Visible, (charges > 0).into());
                VarState::get_mut(entity, world).reindex_statuses();
            }
            charges
        } else {
            GameAssets::get(world)
                .statuses
                .get(name)
                .unwrap()
                .clone()
                .unpack(entity, world);
            VarState::get_mut(entity, world).reindex_statuses();
            Self::change_charges(name, entity, delta, world)
        }
    }
    pub fn get_charges(name: &str, entity: Entity, world: &World) -> Result<i32> {
        Context::new(entity)
            .set_status(name.into())
            .get_int(VarName::Charges, world)
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
                    .is_some_and(|s| {
                        s.get_value_last(VarName::Charges)
                            .and_then(|v| v.get_int())
                            .unwrap_or_default()
                            > 0
                    })
            })
            .collect_vec()
    }
    pub fn notify(event: &Event, context: &Context, world: &mut World) -> bool {
        let owner = context.owner();
        let mut result = false;
        for (status, Status { name, mut trigger }) in Self::collect_active_statuses(owner, world) {
            if context.has_status(owner, name.clone()) {
                continue;
            }
            let context = context.clone().set_status(name).take();
            result |= trigger.fire(event, &context, world);
            world.get_mut::<Status>(status).unwrap().trigger = trigger;
        }
        result
    }
    pub fn map_var(event: &Event, value: &mut VarValue, context: &Context, world: &mut World) {
        let owner = context.owner();
        for (_, Status { name, trigger }) in Self::collect_active_statuses(owner, world) {
            if context.has_status(owner, name.clone()) {
                continue;
            }
            let context = context.clone().set_status(name).take();
            let _ = trigger.change(event, &context, value, world);
        }
    }
    pub fn refresh_mappings(owner: Entity, world: &mut World) {
        let statuses = Self::collect_statuses(owner, world);
        for (_, Status { name, trigger }) in statuses {
            let context = &Context::new(owner).set_status(name.clone()).take();
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

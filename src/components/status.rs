use bevy::ecs::world::EntityMut;

use crate::resourses::event::Event;

use super::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PackedStatus {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub trigger: Trigger,
    #[serde(default)]
    pub representation: Option<Representation>,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub shop_charges: i32,
}

#[derive(Component, Clone)]
pub struct Status {
    pub name: String,
    pub trigger: Trigger,
}

impl PackedStatus {
    pub fn unpack(mut self, owner: Entity, world: &mut World) -> Entity {
        if self.state.get_int(VarName::Charges).is_err() {
            self.state.init(VarName::Charges, VarValue::Int(1));
        }
        self.state
            .init(
                VarName::Description,
                VarValue::String(self.description.to_owned()),
            )
            .init(VarName::Name, VarValue::String(self.name.to_owned()))
            .init(VarName::Position, VarValue::Vec2(default()));
        let add_delta = !self.trigger.collect_delta_triggers().is_empty();
        let entity = Status::spawn_new(self.name, self.trigger, world).id();
        self.state.attach(entity, world);
        if add_delta {
            world.entity_mut(entity).insert(VarStateDelta::default());
        }
        world.entity_mut(entity).set_parent(owner);
        if !SkipVisual::active(world) {
            Options::get_status_rep(world)
                .clone()
                .unpack(Some(entity), None, world);
            if let Some(rep) = self.representation {
                rep.unpack(None, Some(entity), world);
            }
        }
        entity
    }

    pub fn apply_to_team(name: &str, charges: i32, team: &mut PackedTeam) {
        for unit in team.units.iter_mut() {
            if let Some((_, i)) = unit.statuses.iter_mut().find(|(s, _)| s.eq(name)) {
                *i += charges;
            } else {
                unit.statuses.push((name.to_owned(), charges));
            }
        }
    }
}

impl Status {
    pub fn spawn_new(name: String, trigger: Trigger, world: &mut World) -> EntityMut {
        Status { name, trigger }.spawn(world)
    }

    pub fn spawn(self, world: &mut World) -> EntityMut {
        world.spawn((Name::from(self.name.clone()), self))
    }

    pub fn change_charges(
        status: &str,
        unit: Entity,
        delta: i32,
        world: &mut World,
    ) -> Result<Entity> {
        for entity in Self::collect_entity_statuses(unit, world) {
            if let Some(s) = world.entity(entity).get::<Status>() {
                if s.name.eq(status) {
                    VarState::change_int(entity, VarName::Charges, delta, world)?;
                    VarState::push_back(
                        entity,
                        VarName::Visible,
                        VarChange::new(VarValue::Bool(
                            VarState::get(entity, world).get_int(VarName::Charges)? > 0,
                        )),
                        world,
                    );

                    return Ok(entity);
                }
            }
        }
        let mut status = Pools::get_status(status, world).unwrap().clone();
        status.state.init(VarName::Charges, VarValue::Int(delta));
        let entity = status.unpack(unit, world);
        Self::reindex_statuses(unit, world)?;
        Ok(entity)
    }

    fn reindex_statuses(unit: Entity, world: &mut World) -> Result<()> {
        let mut ind: i32 = 0;
        let t = get_insert_head(world);
        for entity in Self::collect_entity_statuses(unit, world) {
            let mut state = VarState::get_mut(entity, world);
            if state.get_int(VarName::Charges).is_ok_and(|x| x > 0) {
                state.insert_simple(VarName::Index, VarValue::Int(ind), t);
                ind += 1;
            }
        }
        Ok(())
    }

    pub fn collect_entity_statuses(entity: Entity, world: &World) -> Vec<Entity> {
        if let Some(entity) = world.get_entity(entity) {
            if let Some(children) = entity.get::<Children>() {
                return children
                    .iter()
                    .copied()
                    .filter(|x| world.entity(*x).contains::<Status>())
                    .collect_vec();
            }
        }
        default()
    }

    pub fn filter_active_statuses(entities: Vec<Entity>, t: f32, world: &World) -> Vec<Entity> {
        entities
            .into_iter()
            .filter(|entity| {
                VarState::find_value(*entity, VarName::Charges, t, world)
                    .is_ok_and(|x| x.get_int().unwrap() > 0)
                    || world.get::<Status>(*entity).unwrap().name.eq(LOCAL_TRIGGER)
            })
            .collect_vec()
    }

    pub fn collect_all_statuses(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<&Children, With<Unit>>()
            .iter(world)
            .collect_vec()
            .into_iter()
            .flat_map(|c| {
                c.into_iter()
                    .filter_map(|e| {
                        if world.get::<Status>(*e).is_some() {
                            Some(*e)
                        } else {
                            None
                        }
                    })
                    .collect_vec()
            })
            .collect_vec()
    }

    pub fn get_trigger(status: Entity, world: &World) -> &Trigger {
        &world.get::<Status>(status).unwrap().trigger
    }

    pub fn collect_event_triggers(
        statuses: Vec<Entity>,
        event: &Event,
        world: &World,
    ) -> Vec<(Entity, Trigger)> {
        statuses
            .into_iter()
            .flat_map(|status| {
                Self::get_trigger(status, world)
                    .catch_event(event)
                    .into_iter()
                    .map(|t| (status, t))
                    .collect_vec()
            })
            .collect_vec()
    }

    pub fn notify(statuses: Vec<Entity>, event: &Event, context: &Context, world: &mut World) {
        for (status, trigger) in Self::collect_event_triggers(statuses, event, world) {
            trigger.fire(event, context, status, world)
        }
    }

    pub fn refresh_entity_mapping(status: Entity, world: &mut World) {
        if let Some(parent) = world.get::<Parent>(status) {
            let parent = parent.get();
            let s = world.get::<Status>(status).unwrap();
            for trigger in s.trigger.collect_delta_triggers() {
                if let Trigger::DeltaVar(var, e) = &trigger {
                    let e = e.clone();
                    let var = *var;
                    if let Ok(delta) = e.get_value(
                        Context::from_owner(parent, world).set_status(status, world),
                        world,
                    ) {
                        let t = get_insert_head(world);
                        let mut state_mapping = world.get_mut::<VarStateDelta>(status).unwrap();
                        if state_mapping.need_update(var, &delta) {
                            state_mapping.state.insert_simple(var, delta, t);
                        }
                    }
                }
            }
        }
    }

    pub fn map_var(
        status: Entity,
        var: VarName,
        value: &mut VarValue,
        context: &Context,
        world: &mut World,
    ) {
        let s = world.get::<Status>(status).unwrap();
        for trigger in s.trigger.collect_map_triggers() {
            debug!("trigger {trigger:?}");
            if let Trigger::MapVar(v, e) = &trigger {
                debug!("Map trigger {v} {e:?}");
                if !var.eq(v) {
                    continue;
                }
                let e = e.clone();
                if let Ok(v) = e.get_value(context.clone().set_var(var, value.clone()), world) {
                    debug!("Value mapped {value:?} into {v:?}");
                    *value = v;
                }
            }
        }
    }
}

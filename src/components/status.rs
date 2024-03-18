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
        let add_delta = self.trigger.has_stat_change();
        let entity = Status::spawn_new(self.name, self.trigger, world).id();
        self.state.attach(entity, world);
        if add_delta {
            world.entity_mut(entity).insert(VarStateDelta::default());
        }
        world.entity_mut(entity).set_parent(owner);
        if !SkipVisual::active(world) {
            if let Some(rep) = self.representation {
                rep.unpack(entity, world);
            } else {
                Options::get_status_rep(world).clone().unpack(entity, world);
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
        for entity in Self::collect_unit_statuses(unit, world) {
            if let Some(s) = world.entity(entity).get::<Status>() {
                if s.name.eq(status) {
                    let mut state =
                        VarState::try_get_mut(entity, world).context("Failed to get state")?;

                    let visible = state.get_bool(VarName::Visible).unwrap_or(true);
                    state.change_int(VarName::Charges, delta);
                    if visible != (state.get_int(VarName::Charges)? > 0) {
                        state.push_back(VarName::Visible, VarChange::new(VarValue::Bool(!visible)));
                    }

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
        let t = GameTimer::get().insert_head();
        for entity in Self::collect_unit_statuses(unit, world) {
            let mut state = VarState::get_mut(entity, world);
            if state.get_int(VarName::Charges).is_ok_and(|x| x > 0) {
                state.insert_simple(VarName::StatusIndex, VarValue::Int(ind), t);
                ind += 1;
            }
        }
        Ok(())
    }

    pub fn collect_statuses_name_charges(
        entity: Entity,
        t: f32,
        world: &World,
    ) -> Vec<(String, i32)> {
        Self::collect_unit_statuses(entity, world)
            .into_iter()
            .filter_map(|entity| {
                let state = VarState::snapshot(entity, world, t);
                let charges = state.get_int(VarName::Charges);
                if charges.is_err() || *charges.as_ref().unwrap() <= 0 {
                    return None;
                }
                match state.get_string(VarName::Name) {
                    Ok(name) => Some((name, charges.unwrap())),
                    Err(_) => None,
                }
            })
            .collect_vec()
    }

    pub fn collect_unit_statuses(unit: Entity, world: &World) -> Vec<Entity> {
        if let Some(entity) = world.get_entity(unit) {
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

    pub fn get_status_charges(unit: Entity, status: &str, world: &World) -> Result<i32> {
        Self::collect_unit_statuses(unit, world)
            .into_iter()
            .find_map(|e| {
                let state = VarState::try_get(e, world).ok()?;
                let name = state.get_string(VarName::Name).ok()?;
                match name.eq(status) {
                    true => Some(state.get_int(VarName::Charges).ok()?),
                    false => None,
                }
            })
            .with_context(|| format!("Failed to find status {status} for {unit:?}"))
    }

    pub fn find_unit_status<'a>(
        unit: Entity,
        name: &str,
        world: &'a mut World,
    ) -> Option<Mut<'a, Status>> {
        if let Some(entity) = world.get_entity(unit) {
            if let Some(children) = entity.get::<Children>() {
                if let Some(status) = children.iter().copied().find_map(|x| {
                    if world.get::<Status>(x).is_some_and(|s| s.name.eq(name)) {
                        Some(x)
                    } else {
                        None
                    }
                }) {
                    return Some(world.get_mut::<Status>(status).unwrap());
                }
            }
        }
        None
    }

    pub fn filter_active_statuses(entities: Vec<Entity>, t: f32, world: &World) -> Vec<Entity> {
        entities
            .into_iter()
            .filter(|entity| {
                VarState::find_value(*entity, VarName::Charges, t, world)
                    .is_ok_and(|x| x.get_int().unwrap() > 0)
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

    pub fn collect_triggers(statuses: Vec<Entity>, world: &World) -> Vec<(Entity, Trigger)> {
        statuses
            .into_iter()
            .map(|status| (status, Self::get_trigger(status, world).clone()))
            .collect_vec()
    }

    pub fn notify(
        statuses: Vec<Entity>,
        event: &Event,
        context: &Context,
        world: &mut World,
    ) -> bool {
        let mut result = false;
        for (status, trigger) in Self::collect_triggers(statuses, world) {
            if context.has_status(status) {
                continue;
            }
            result |= trigger.fire(
                event,
                context
                    .clone()
                    .set_owner(status.get_parent(world).unwrap(), world)
                    .set_status(status, world),
                world,
            );
        }
        result
    }

    pub fn refresh_status_mapping(status: Entity, world: &mut World) {
        if let Some(parent) = world.get::<Parent>(status) {
            let parent = parent.get();
            let context = Context::from_owner(parent, world)
                .set_status(status, world)
                .take();
            let s = world.get::<Status>(status).unwrap();
            for (var, value) in s.trigger.clone().collect_mappings(&context, world) {
                let t = GameTimer::get().insert_head();
                let mut state_mapping = world.get_mut::<VarStateDelta>(status).unwrap();
                if state_mapping.need_update(var, &value) {
                    state_mapping.state.insert_simple(var, value, t);
                }
            }
        }
    }

    pub fn map_var(
        status: Entity,
        event: &Event,
        value: &mut VarValue,
        context: &Context,
        world: &mut World,
    ) {
        let s = world.get::<Status>(status).unwrap();
        let _ = s.trigger.clone().change(event, context, value, world);
    }
}

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
    pub representation: Option<Representation>,
    #[serde(default)]
    pub state: VarState,
}

#[derive(Component)]
pub struct Status {
    pub name: String,
    pub trigger: Trigger,
}

impl PackedStatus {
    pub fn unpack(mut self, owner: Option<Entity>, world: &mut World) -> Result<Entity> {
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
        let add_delta = match &self.trigger {
            Trigger::ChangeVar(_, _) => true,
            _ => false,
        };
        let entity = Status::spawn(self.name, self.trigger, world)
            .insert(self.state)
            .id();
        if add_delta {
            world.entity_mut(entity).insert(VarStateDelta::default());
        }
        self.representation
            .unwrap()
            .unpack(Some(entity), owner, world);
        Ok(entity)
    }
}

impl Status {
    pub fn spawn(name: String, trigger: Trigger, world: &mut World) -> EntityMut {
        world.spawn((Name::from(name.clone()), Status { name, trigger }))
    }

    pub fn change_charges(
        status: &str,
        unit: Entity,
        delta: i32,
        world: &mut World,
    ) -> Result<Entity> {
        for entity in Self::collect_entity_statuses(unit, world) {
            if let Some(s) = world.entity_mut(entity).get_mut::<Status>() {
                if s.name.eq(status) {
                    VarState::change_int(entity, VarName::Charges, delta, world)?;
                    return Ok(entity);
                }
            }
        }
        let mut status = Pools::get_status(status, world).clone();
        status.state.init(VarName::Charges, VarValue::Int(delta));
        status.unpack(Some(unit), world)
    }

    pub fn collect_entity_statuses(entity: Entity, world: &World) -> Vec<Entity> {
        if let Some(entity) = world.get_entity(entity) {
            if let Some(children) = entity.get::<Children>() {
                return children
                    .to_vec()
                    .into_iter()
                    .filter(|x| world.entity(*x).contains::<Status>())
                    .collect_vec();
            }
        }
        default()
    }

    pub fn collect_all_statuses(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<Entity, With<Status>>()
            .iter(world)
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
            .filter_map(|status| {
                Self::get_trigger(status, world)
                    .catch_event(event)
                    .map(|t| (status, t))
            })
            .collect_vec()
    }

    pub fn notify(statuses: Vec<Entity>, event: &Event, context: &Context, world: &mut World) {
        for (status, trigger) in Self::collect_event_triggers(statuses, event, world) {
            trigger.fire(event, context, status, world)
        }
    }
}

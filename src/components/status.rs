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
    pub fn unpack(mut self, entity: Option<Entity>, world: &mut World) -> Result<Entity> {
        let entity = self.representation.unwrap().unpack(entity, world);
        if self.state.get_int(VarName::Charges).is_err() {
            self.state.insert(VarName::Charges, VarValue::Int(1));
        }
        self.state
            .insert(
                VarName::Description,
                VarValue::String(self.description.to_owned()),
            )
            .insert(VarName::Name, VarValue::String(self.name.to_owned()))
            .insert(VarName::Position, VarValue::Vec2(default()));
        world
            .get_entity_mut(entity)
            .context("Entity doesn't exist")?
            .insert((
                Status {
                    name: self.name.clone(),
                    trigger: self.trigger,
                },
                Name::from(self.name),
                self.state,
            ));
        Ok(entity)
    }
}

impl Status {
    pub fn change_charges(
        status_name: &str,
        unit: Entity,
        delta: i32,
        world: &mut World,
    ) -> Result<Entity> {
        for entity in Self::collect_all_statuses(unit, world) {
            if let Some(status) = world.entity_mut(entity).get_mut::<Status>() {
                if status.name.eq(status_name) {
                    VarState::change_int(entity, VarName::Charges, delta, world)?;
                    return Ok(entity);
                }
            }
        }
        let mut status = Options::get_statuses(world)
            .get(status_name)
            .unwrap()
            .clone();
        status.state.insert(VarName::Charges, VarValue::Int(delta));
        status.unpack(Some(unit), world)
    }

    pub fn collect_all_statuses(entity: Entity, world: &World) -> Vec<Entity> {
        if let Some(entity) = world.get_entity(entity) {
            entity
                .get::<Children>()
                .unwrap()
                .to_vec()
                .into_iter()
                .filter(|x| world.entity(*x).contains::<Status>())
                .collect_vec()
        } else {
            default()
        }
    }

    pub fn get_trigger(status: Entity, world: &World) -> &Trigger {
        &world.get::<Status>(status).unwrap().trigger
    }

    pub fn notify(statuses: Vec<Entity>, event: &Event, world: &mut World) {
        for status in statuses {
            Self::get_trigger(status, world)
                .clone()
                .catch_event(event, status, world)
                .unwrap();
        }
    }
}

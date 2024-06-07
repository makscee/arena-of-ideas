use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Corpse;

impl UnitPlugin {
    pub fn run_death_check(world: &mut World) -> Vec<Entity> {
        let dead = world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .filter(|e| Self::is_dead(*e, world))
            .collect_vec();
        // for unit in dead.iter() {
        //     Self::send_death_events(*unit, world);
        // }
        dead
    }
    pub fn is_dead(entity: Entity, world: &World) -> bool {
        let context = Context::new(entity);
        context
            .get_var(VarName::Hp, world)
            .unwrap()
            .get_int()
            .unwrap()
            <= context
                .get_var(VarName::Dmg, world)
                .map(|v| v.get_int().unwrap_or_default())
                .unwrap_or_default()
    }
    pub fn turn_into_corpse(entity: Entity, world: &mut World) {
        debug!("Turn {entity:?} into corpse");
        let mut unit = world.entity_mut(entity);
        unit.remove::<Unit>();
        unit.insert(Corpse);
        VarState::get_mut(unit.id(), world).push_change(
            VarName::Visible,
            default(),
            VarChange::new(VarValue::Bool(false)),
        );
    }
    pub fn collect_faction(faction: Faction, world: &mut World) -> Vec<Entity> {
        if let Some(children) = world.get::<Children>(TeamPlugin::entity(faction, world)) {
            return children
                .iter()
                .filter_map(|e| world.get::<Unit>(*e).map(|_| *e))
                .collect_vec();
        }
        default()
    }
    pub fn find_unit(faction: Faction, slot: i32, world: &mut World) -> Option<Entity> {
        Self::collect_faction(faction, world).into_iter().find(|e| {
            VarState::get(*e, world).get_int(VarName::Slot).unwrap() == slot
                && !Self::is_dead(*e, world)
        })
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Display,
    Default,
    AsRefStr,
    EnumString,
    EnumIter,
)]
pub enum Faction {
    #[default]
    Left,
    Right,
    Team,
    Shop,
}

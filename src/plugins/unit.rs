use bevy::ecs::query::Or;

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
    pub fn collect_all(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<Entity, Or<(With<Unit>, With<Corpse>)>>()
            .iter(world)
            .collect_vec()
    }
    pub fn collect_alive(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .collect_vec()
    }
    pub fn collect_faction(faction: Faction, world: &World) -> Vec<Entity> {
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
    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        Self::make_slot_gap(faction, i32::MAX, world);
    }
    pub fn make_slot_gap(faction: Faction, slot: i32, world: &mut World) {
        for (unit, ind) in Self::collect_faction(faction, world)
            .into_iter()
            .sorted_by_key(|x| VarState::get(*x, world).get_int(VarName::Slot).unwrap())
            .zip(1..)
            .collect_vec()
        {
            let slot = ind + if ind >= slot { 1 } else { 0 };
            VarState::get_mut(unit, world).init(VarName::Slot, VarValue::Int(slot));
        }
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

impl Faction {
    pub fn opposite(&self) -> Self {
        match self {
            Faction::Left => Faction::Right,
            Faction::Right => Faction::Left,
            _ => panic!("Can't get opposite of {self}"),
        }
    }
}

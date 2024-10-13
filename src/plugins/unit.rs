use bevy::app::PreUpdate;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, TeamContainer::place_into_slots);
    }
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
        for unit in dead.iter() {
            Self::send_death_events(*unit, world);
        }
        dead
    }
    fn send_death_events(entity: Entity, world: &mut World) {
        Event::Death(entity).send(world);
        if let Ok(killer) = Context::new(entity).get_entity(VarName::LastAttacker, world) {
            Event::Kill {
                owner: killer,
                target: entity,
            }
            .send(world);
        }
    }
    pub fn is_dead(entity: Entity, world: &World) -> bool {
        let context = Context::new(entity);
        context
            .get_value(VarName::Hp, world)
            .unwrap()
            .get_int()
            .unwrap()
            <= context
                .get_value(VarName::Dmg, world)
                .map(|v| v.get_int().unwrap_or_default())
                .unwrap_or_default()
    }
    pub fn turn_into_corpse(entity: Entity, world: &mut World) {
        debug!("Turn {entity} into corpse");
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
    pub fn collect_factions(
        factions: HashSet<Faction>,
        world: &mut World,
    ) -> Vec<(Entity, Faction)> {
        factions
            .into_iter()
            .flat_map(|f| {
                Self::collect_faction(f, world)
                    .into_iter()
                    .map(move |e| (e, f))
            })
            .collect_vec()
    }
    pub fn find_unit(faction: Faction, slot: i32, world: &mut World) -> Option<Entity> {
        Self::collect_faction(faction, world).into_iter().find(|e| {
            Context::new(*e).get_int(VarName::Slot, world).unwrap() == slot
                && !Self::is_dead(*e, world)
        })
    }
    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        Self::make_slot_gap(faction, i32::MAX, world);
    }
    pub fn fill_gaps_and_translate(world: &mut World) {
        Self::fill_slot_gaps(Faction::Left, world);
        Self::fill_slot_gaps(Faction::Right, world);
        Self::translate_to_slots(world);
    }
    pub fn make_slot_gap(faction: Faction, slot: i32, world: &mut World) {
        for (unit, ind) in Self::collect_faction(faction, world)
            .into_iter()
            .sorted_by_key(|x| Context::new(*x).get_int(VarName::Slot, world).unwrap())
            .zip(0..)
            .collect_vec()
        {
            let slot = ind + if ind >= slot { 1 } else { 0 };
            VarState::get_mut(unit, world).init(VarName::Slot, VarValue::Int(slot));
        }
    }
    pub fn get_unit_position(entity: Entity, world: &World) -> Result<Vec2> {
        Context::new(entity).get_vec2(VarName::Position, world)
    }
    pub fn get_entity_slot_position(entity: Entity, world: &World) -> Result<Vec2> {
        let context = Context::new(entity);
        let slot = context.get_value(VarName::Slot, world)?.get_int()? as usize;
        let faction = context.get_value(VarName::Faction, world)?.get_faction()?;
        Ok(TeamContainer::slot_position(faction, slot, world))
    }
    pub fn translate_to_slots(world: &mut World) {
        let mut shift: f32 = 0.0;
        for unit in Self::collect_all(world) {
            shift = shift.max(
                GameAssets::get(world)
                    .animations
                    .move_to_slot
                    .clone()
                    .apply(Context::new(unit), world)
                    .unwrap(),
            );
        }
        gt().advance_insert(shift);
    }
    pub fn despawn(entity: Entity, world: &mut World) {
        world.entity_mut(entity).despawn_recursive();
    }
    pub fn name_cstr(name: &str) -> Cstr {
        let bases = name.split("+").collect_vec();
        Self::name_from_bases(bases, 3)
    }
    pub fn name_from_bases(bases: Vec<&str>, max_chars: usize) -> Cstr {
        if bases.len() == 1 && max_chars != 1 {
            let name = bases[0];
            return name.cstr_c(name_color(name));
        }
        let mut name = Cstr::default();
        let max_chars = if max_chars == 0 {
            usize::MAX
        } else {
            max_chars
        };
        for (i, n) in bases.iter().enumerate() {
            let c = name_color(n);
            let n = bases[i];
            let n = if i == 0 {
                n.split_at(n.len() / 2).0
            } else if i == bases.len() - 1 {
                n.split_at(n.len() / 2).1
            } else {
                n.split_at(n.len() / 2).1
            };
            let n = n.split_at(n.len().min(max_chars)).0;
            name.push(n.cstr_c(c));
        }
        name
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

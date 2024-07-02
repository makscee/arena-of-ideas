use bevy::app::PreUpdate;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::place_into_slots);
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
            VarState::get(*e, world).get_int(VarName::Slot).unwrap() == slot
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
            .sorted_by_key(|x| VarState::get(*x, world).get_int(VarName::Slot).unwrap())
            .zip(1..)
            .collect_vec()
        {
            let slot = ind + if ind >= slot { 1 } else { 0 };
            VarState::get_mut(unit, world).init(VarName::Slot, VarValue::Int(slot));
        }
    }
    pub fn get_slot_position(faction: Faction, slot: usize) -> Vec2 {
        match faction {
            Faction::Left => vec2(slot as f32 * -SLOT_SPACING, 0.0),
            Faction::Right => vec2(slot as f32 * SLOT_SPACING, 0.0),
            Faction::Team => vec2(slot as f32 * -SLOT_SPACING + 14.0, -3.0),
            Faction::Shop => vec2(slot as f32 * SLOT_SPACING - 4.0, 5.5),
        }
    }
    pub fn get_entity_slot_position(entity: Entity, world: &World) -> Result<Vec2> {
        let context = Context::new(entity);
        let slot = context.get_var(VarName::Slot, world)?.get_int()? as usize;
        let faction = context.get_var(VarName::Faction, world)?.get_faction()?;
        Ok(Self::get_slot_position(faction, slot))
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
        GameTimer::get().advance_insert(shift);
    }
    fn place_into_slots(world: &mut World) {
        let Some(cam_entity) = CameraPlugin::get_entity(world) else {
            return;
        };
        let delta = delta_time(world);
        let units = Self::collect_factions([Faction::Shop, Faction::Team].into(), world);
        let data = world.remove_resource::<WidgetData>().unwrap();
        let camera = world.get::<Camera>(cam_entity).unwrap().clone();
        let transform = world.get::<GlobalTransform>(cam_entity).unwrap().clone();
        for (entity, faction) in units {
            let mut state = VarState::get_mut(entity, world);
            let slot = state.get_int(VarName::Slot).unwrap();
            let position = state.get_vec2(VarName::Position).unwrap();
            let need_pos = data
                .unit_container
                .get(&faction)
                .and_then(|d| d.positions.get(slot as usize))
                .map(|p| screen_to_world_cam(p.to_bvec2(), &camera, &transform))
                .unwrap_or_default();
            state.change_vec2(VarName::Position, (need_pos - position) * delta * 5.0);
        }
        world.insert_resource(data);
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

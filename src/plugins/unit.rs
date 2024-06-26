use super::*;
use crate::module_bindings::{once_on_run_stack, run_stack, run_team_reorder};
use resources::event::Event;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredUnit>()
            .init_resource::<DraggedUnit>()
            .add_systems(Update, Self::ui);
    }
}

pub const TEAM_SLOTS: usize = 7;
pub const SLOT_SPACING: f32 = 3.0;
pub const UNITS_LIMIT: usize = 50;

#[derive(Resource, Debug, Copy, Clone)]
pub struct ClosestSlot(usize, f32, bool);

impl UnitPlugin {
    pub fn get_unit_position(entity: Entity, world: &World) -> Result<Vec2> {
        VarState::get(entity, world)
            .get_value_last(VarName::Position)?
            .get_vec2()
    }

    pub fn find_unit(faction: Faction, slot: usize, world: &mut World) -> Option<Entity> {
        Self::collect_faction(faction, world).into_iter().find(|e| {
            VarState::get(*e, world).get_int(VarName::Slot).unwrap() == slot as i32
                && !Self::is_dead(*e, world)
        })
    }

    pub fn get_id(entity: Entity, world: &World) -> Option<u64> {
        VarState::try_get(entity, world)
            .and_then(|state| state.get_int(VarName::Id).map(|v| v as u64))
            .ok()
    }
    pub fn get_by_id(id: u64, world: &mut World) -> Option<Entity> {
        Self::collect_all(world).into_iter().find_map(|unit| {
            VarState::try_get(unit, world).ok().and_then(|s| {
                s.get_int(VarName::Id).ok().and_then(|tid| {
                    if tid as u64 == id {
                        Some(unit)
                    } else {
                        None
                    }
                })
            })
        })
    }

    pub fn get_slot_position(faction: Faction, slot: usize) -> Vec2 {
        match faction {
            Faction::Left => vec2(slot as f32 * -SLOT_SPACING, 0.0),
            Faction::Right => vec2(slot as f32 * SLOT_SPACING, 0.0),
            Faction::Team => vec2(slot as f32 * -SLOT_SPACING + 14.0, -3.0),
            Faction::Shop => vec2(slot as f32 * SLOT_SPACING - 4.0, 5.5),
        }
    }

    pub fn get_closest_slot(pos: Vec2, faction: Faction) -> (usize, f32) {
        let mut closest_slot = (0, f32::MAX);
        for slot in 0..=TEAM_SLOTS {
            let dist = (Self::get_slot_position(faction, slot).x - pos.x).abs();
            if dist < closest_slot.1 {
                closest_slot = (slot, dist);
            }
        }
        closest_slot
    }

    pub fn get_entity_slot_position(entity: Entity, world: &World) -> Result<Vec2> {
        let context = Context::from_owner(entity, world);
        let slot = context.get_var(VarName::Slot, world)?.get_int()? as usize;
        let faction = context.get_var(VarName::Faction, world)?.get_faction()?;
        Ok(Self::get_slot_position(faction, slot))
    }

    pub fn collect_all(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .collect_vec()
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

    pub fn collect_faction(faction: Faction, world: &mut World) -> Vec<Entity> {
        if let Some(team) = TeamPlugin::find_entity(faction, world) {
            if let Some(children) = world.get::<Children>(team) {
                return children
                    .iter()
                    .filter_map(|e| world.get::<Unit>(*e).map(|_| *e))
                    .collect_vec();
            }
        }
        default()
    }

    pub fn collect_unit_faction(unit: Entity, world: &World) -> Vec<Entity> {
        if let Some(team) = unit.get_parent(world) {
            if let Some(children) = world.get::<Children>(team) {
                return children
                    .iter()
                    .filter_map(|e| world.get::<Unit>(*e).map(|_| *e))
                    .collect_vec();
            }
        }
        default()
    }

    pub fn collect_faction_ids(faction: Faction, world: &mut World) -> HashMap<u64, Entity> {
        HashMap::from_iter(Self::collect_faction(faction, world).into_iter().map(|u| {
            (
                VarState::get(u, world).get_int(VarName::Id).unwrap() as u64,
                u,
            )
        }))
    }

    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        Self::make_slot_gap(faction, i32::MAX, world);
    }

    pub fn make_slot_gap(faction: Faction, slot: i32, world: &mut World) {
        let dragged = world.resource::<DraggedUnit>().0;
        for (unit, ind) in Self::collect_faction(faction, world)
            .into_iter()
            .filter(|u| match dragged {
                Some((d, _)) => d != *u,
                None => true,
            })
            .sorted_by_key(|x| VarState::get(*x, world).get_int(VarName::Slot).unwrap())
            .zip(1..)
            .collect_vec()
        {
            let slot = ind + if ind >= slot { 1 } else { 0 };
            VarState::get_mut(unit, world).init(VarName::Slot, VarValue::Int(slot));
        }
    }

    pub fn place_into_slot(entity: Entity, world: &mut World) -> Result<()> {
        let context = Context::from_owner(entity, world);
        let faction = context.get_var(VarName::Faction, world)?.get_faction()?;
        let slot = context.get_var(VarName::Slot, world)?.get_int()?;
        let pos = Self::get_slot_position(faction, slot as usize);
        VarState::get_mut(entity, world)
            .push_back(VarName::Position, VarChange::new(VarValue::Vec2(pos)));
        Ok(())
    }

    pub fn translate_unit(entity: Entity, position: Vec2, world: &mut World) {
        VarState::get_mut(entity, world).push_back(
            VarName::Position,
            VarChange::new(VarValue::Vec2(position))
                .set_duration(0.3)
                .set_tween(Tween::QuartOut),
        );
    }

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
        if let Ok(killer) = VarState::get(entity, world).get_entity(VarName::LastAttacker) {
            Event::Kill {
                owner: killer,
                target: entity,
            }
            .send(world);
        }
    }

    pub fn turn_into_corpse(entity: Entity, world: &mut World) {
        debug!("Turn {entity:?} into corpse");
        let mut unit = world.entity_mut(entity);
        unit.remove::<Unit>();
        unit.insert(Corpse);
        VarState::get_mut(unit.id(), world)
            .push_back(VarName::Visible, VarChange::new(VarValue::Bool(false)));
    }

    pub fn is_dead(entity: Entity, world: &World) -> bool {
        let context = Context::from_owner(entity, world);
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

    pub fn despawn_all_teams(world: &mut World) {
        debug!("Called despawn all teams");
        for team in world
            .query_filtered::<Entity, With<Team>>()
            .iter(world)
            .collect_vec()
        {
            world.entity_mut(team).despawn_recursive()
        }
    }

    pub fn despawn_all_units(world: &mut World) {
        debug!("Called despawn all units");
        for entity in world
            .query_filtered::<Entity, Or<(With<Unit>, With<Corpse>)>>()
            .iter(world)
            .collect_vec()
        {
            world.entity_mut(entity).despawn_recursive();
        }
    }

    pub fn translate_to_slots(world: &mut World) {
        let units = UnitPlugin::collect_factions(
            HashSet::from([Faction::Left, Faction::Right, Faction::Team, Faction::Shop]),
            world,
        );
        GameTimer::get().start_batch();
        for (unit, faction) in units.into_iter() {
            let slot = VarState::get(unit, world).get_int(VarName::Slot).unwrap() as usize;
            UnitPlugin::translate_unit(unit, UnitPlugin::get_slot_position(faction, slot), world);
            GameTimer::get().to_batch_start();
        }
        GameTimer::get().end_batch();
    }

    pub fn fill_gaps_and_translate(world: &mut World) {
        Self::fill_slot_gaps(Faction::Left, world);
        Self::fill_slot_gaps(Faction::Right, world);
        Self::translate_to_slots(world);
    }
    pub fn fill_gaps_and_place(world: &mut World) {
        Self::fill_slot_gaps(Faction::Left, world);
        Self::fill_slot_gaps(Faction::Right, world);
        for (unit, _) in Self::collect_factions([Faction::Left, Faction::Right].into(), world) {
            Self::place_into_slot(unit, world).unwrap();
        }
    }

    pub fn get_hovered(world: &World) -> Option<Entity> {
        world
            .resource::<HoveredUnit>()
            .0
            .and_then(|e| e.get_parent(world))
    }

    pub fn hover_unit(
        event: Listener<Pointer<Move>>,
        mut hovered: ResMut<HoveredUnit>,
        parent: Query<&Parent>,
    ) {
        hovered.0 = event.target.get_parent_query(&parent);
    }

    pub fn unhover_unit(_event: Listener<Pointer<Out>>, mut hovered: ResMut<HoveredUnit>) {
        hovered.0 = None;
    }

    pub fn drag_unit_start(
        event: Listener<Pointer<DragStart>>,
        mut team: Query<Entity, With<ActiveTeam>>,
        mut dragged: ResMut<DraggedUnit>,
        mut commands: Commands,
        parent: Query<&Parent>,
        shop_data: Res<ShopData>,
    ) {
        if !shop_data.is_initial_phase() {
            return;
        }
        let entity: Entity = event.target;
        let entity = entity
            .get_parent_query(&parent)
            .unwrap()
            .get_parent_query(&parent)
            .unwrap();
        debug!("Drag unit start {:?}", entity);
        dragged.0 = Some((entity, DragAction::None));
        for entity in team.iter_mut() {
            commands.entity(entity).insert(Pickable::IGNORE);
        }
        commands.insert_resource(ClosestSlot(0, f32::MAX, false));
    }

    pub fn drag_unit_end(
        event: Listener<Pointer<DragEnd>>,
        mut team: Query<Entity, With<ActiveTeam>>,
        mut commands: Commands,
    ) {
        debug!("Drag unit end {:?}", event.target);
        for entity in team.iter_mut() {
            commands.entity(entity).insert(Pickable::default());
        }

        commands.add(|world: &mut World| {
            if let Some((dragged, action)) = world.resource::<DraggedUnit>().0 {
                world.resource_mut::<DraggedUnit>().0 = None;
                match action {
                    DragAction::Fuse(target) => {
                        ShopPlugin::start_fuse(target, dragged, world);
                    }
                    DragAction::Stack(target) => Self::stack_units(target, dragged, world),
                    DragAction::Insert(_) | DragAction::None => {
                        let sorted_ids = UnitPlugin::collect_faction_ids(Faction::Team, world)
                            .into_iter()
                            .map(|(id, entity)| {
                                (
                                    id,
                                    VarState::get(entity, world).get_int(VarName::Slot).unwrap(),
                                )
                            })
                            .sorted_by_key(|(_, s)| *s)
                            .map(|(id, _)| id)
                            .collect_vec();
                        run_team_reorder(sorted_ids);
                    }
                    DragAction::Sell => todo!(),
                }
                Self::fill_slot_gaps(Faction::Team, world);
                Self::fill_slot_gaps(Faction::Shop, world);
                Self::translate_to_slots(world);
            }
        });
    }

    pub fn drag_unit(world: &mut World) {
        if let Some((dragged, action)) = world.resource::<DraggedUnit>().0 {
            if let Some(cursor_pos) = CameraPlugin::cursor_world_pos(world) {
                let mut transform = world.get_mut::<Transform>(dragged).unwrap();
                transform.translation = cursor_pos.extend(0.0);
                VarState::get_mut(dragged, world)
                    .init(VarName::Position, VarValue::Vec2(cursor_pos));
            }
            if let DragAction::Insert(slot) = action {
                let mut state = VarState::get_mut(dragged, world);
                let slot = slot as i32;
                if state.get_int(VarName::Slot).is_ok_and(|s| s != slot) {
                    state.init(VarName::Slot, VarValue::Int(slot));
                    UnitPlugin::make_slot_gap(Faction::Team, slot, world);
                    UnitPlugin::translate_to_slots(world);
                }
            }
        }
    }

    fn ui(world: &mut World) {
        let hovered = UnitPlugin::get_hovered(world);
        if let Some(sd) = world.get_resource::<ShopData>() {
            if !sd.show_other_ui() {
                return;
            }
        }
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let show_card = SettingsData::get(world).always_show_card;
        let dragging = world.resource::<DraggedUnit>().0.is_some();
        for (entity, state) in world
            .query_filtered::<(Entity, &VarState), Or<(With<Unit>, With<Corpse>)>>()
            .iter(world)
        {
            let t = GameTimer::get().play_head();
            if !state
                .get_value_at(VarName::Visible, t)
                .and_then(|v| v.get_bool())
                .unwrap_or(true)
            {
                continue;
            }
            if let Some(text) = world.get::<TextColumn>(entity) {
                text.render(ctx, world);
            }
            let statuses = Status::collect_statuses_name_charges(entity, None, t, world);
            state.show_entity_card_window(
                entity,
                statuses,
                (show_card || hovered == Some(entity)) && !dragging,
                ctx,
                world,
            );
        }
    }

    pub fn get_faction(unit: Entity, world: &World) -> Faction {
        Context::from_owner(unit, world)
            .get_var(VarName::Faction, world)
            .unwrap()
            .get_faction()
            .unwrap()
    }

    pub fn collect_stack_targets(world: &mut World) -> HashMap<Entity, Entity> {
        let units = HashMap::from_iter(UnitPlugin::collect_factions(
            [Faction::Team, Faction::Shop].into(),
            world,
        ));
        let names = HashMap::from_iter(units.clone().into_iter().map(|(u, _)| {
            (
                u,
                VarState::get(u, world).get_string(VarName::Name).unwrap(),
            )
        }));
        HashMap::from_iter(units.clone().into_iter().filter_map(|(u, _)| {
            let name = names.get(&u).unwrap();
            names.iter().find_map(|(e, n)| {
                match name.eq(n) && !e.eq(&u) && Faction::Team.eq(units.get(e).unwrap()) {
                    true => Some((u, *e)),
                    false => None,
                }
            })
        }))
    }

    fn filter_unit_stack_targets(
        unit: Entity,
        targets: &Vec<Entity>,
        world: &World,
    ) -> Vec<Entity> {
        let state = VarState::get(unit, world);
        let name = state.get_string(VarName::Name).unwrap();
        let houses = state.get_houses_vec().unwrap();
        targets
            .clone()
            .into_iter()
            .filter(|target| {
                !unit.eq(target) && {
                    let state = VarState::get(*target, world);
                    let target_houses = state.get_houses_vec().unwrap();
                    state.get_string(VarName::Name).unwrap().eq(&name)
                        || (target_houses.len() > 1 || state.get_int(VarName::Level).unwrap() > 1)
                            && houses.iter().any(|h| target_houses.contains(h))
                }
            })
            .collect_vec()
    }
    fn filter_unit_fuse_targets(unit: Entity, targets: &Vec<Entity>, world: &World) -> Vec<Entity> {
        let state = VarState::get(unit, world);
        if !state.is_fuse_source().unwrap() {
            return default();
        }
        targets
            .clone()
            .into_iter()
            .filter(|target| {
                !unit.eq(target) && VarState::get(*target, world).is_fuse_target(state).unwrap()
            })
            .collect_vec()
    }

    pub fn collect_merge_targets(
        sources: Vec<Entity>,
        targets: Vec<Entity>,
        world: &World,
    ) -> (HashMap<Entity, Vec<Entity>>, HashMap<Entity, Vec<Entity>>) {
        let mut fuse: HashMap<Entity, Vec<Entity>> = default();
        let mut stack: HashMap<Entity, Vec<Entity>> = default();
        for unit in sources {
            let stack_targets = Self::filter_unit_stack_targets(unit, &targets, world);
            let fuse_targets = Self::filter_unit_fuse_targets(unit, &targets, world);
            if !stack_targets.is_empty() {
                stack.insert(unit, stack_targets);
            }
            if !fuse_targets.is_empty() {
                fuse.insert(unit, fuse_targets);
            }
        }
        (stack, fuse)
    }

    pub fn stack_units(target: Entity, source: Entity, world: &mut World) {
        let target_id = Self::get_id(target, world).unwrap();
        let source_id = Self::get_id(source, world).unwrap();
        run_stack(target_id, source_id);
        once_on_run_stack(move |_, _, s, target, _| match s {
            spacetimedb_sdk::reducer::Status::Committed => {
                let target = *target;
                OperationsPlugin::add(move |world| {
                    if let Some(target) = Self::get_by_id(target, world) {
                        TextColumn::add(target, "+Stack", yellow(), world).unwrap();
                    }
                })
            }
            spacetimedb_sdk::reducer::Status::Failed(e) => {
                AlertPlugin::add_error(Some("Stack error".to_owned()), e.to_string(), None)
            }
            spacetimedb_sdk::reducer::Status::OutOfEnergy => panic!(),
        });
    }

    pub fn spawn_slot(slot: usize, faction: Faction, world: &mut World) {
        let pos = UnitPlugin::get_slot_position(Faction::Team, slot);
        if let Some(team) = TeamPlugin::find_entity(faction, world) {
            let entity = world.spawn_empty().set_parent(team).id();
            let rep = Options::get_slot_rep(world).clone().unpack(entity, world);
            let mut state = VarState::new_with(VarName::Position, VarValue::Vec2(pos));
            state.init(VarName::Slot, VarValue::Int(slot as i32));
            state.attach(entity, world);
            world.get_mut::<Transform>(rep).unwrap().translation.z -= 100.0;
            world.entity_mut(rep).insert(Slot);
        }
    }
}

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct Slot;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Reflect,
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
    pub fn team_entity(&self, world: &mut World) -> Entity {
        TeamPlugin::find_entity(*self, world).unwrap_or_else(|| TeamPlugin::spawn(*self, world))
    }
}

#[derive(Resource, Default)]
pub struct HoveredUnit(pub Option<Entity>);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DraggedUnit(pub Option<(Entity, DragAction)>);

#[derive(Debug, Copy, Clone)]
pub enum DragAction {
    None,
    Fuse(Entity),
    Stack(Entity),
    Insert(usize),
    Sell,
}

#[derive(Component)]
pub struct ActiveTeam;

impl VarState {
    pub fn is_fuse_source(&self) -> Result<bool> {
        let level = self.get_int(VarName::Level)?;
        let houses = self.get_houses_vec()?;
        Ok(level > 1 && houses.len() == 1)
    }
    pub fn is_fuse_target(&self, source: &VarState) -> Result<bool> {
        let level = self.get_int(VarName::Level)?;
        let source_level = source.get_int(VarName::Level)?;
        let houses = self.get_houses_vec()?;
        let level_req = houses.len() as i32 + 1;
        Ok(level >= level_req
            && source_level >= level_req
            && !houses.contains(&source.get_houses_vec()?.into_iter().exactly_one().unwrap()))
    }
}

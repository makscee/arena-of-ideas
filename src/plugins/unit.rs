use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredUnit>()
            .init_resource::<DraggedUnit>()
            .add_systems(Update, Self::ui);
    }
}

pub const TEAM_SLOTS: usize = 7;

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

    pub fn get_slot_position(faction: Faction, slot: usize) -> Vec2 {
        match faction {
            Faction::Left => vec2(slot as f32 * -3.0, 0.0),
            Faction::Right => vec2(slot as f32 * 3.0, 0.0),
            Faction::Team => vec2(slot as f32 * -3.0 + 14.0, -4.0),
            Faction::Shop => vec2(slot as f32 * 3.0 - 4.0, 4.0),
        }
    }

    pub fn get_closest_slot(pos: Vec2, faction: Faction) -> (usize, f32) {
        let mut closest_slot = (0, f32::MAX);
        for slot in 0..TEAM_SLOTS {
            let dist = (Self::get_slot_position(faction, slot).x - pos.x).abs();
            if dist < closest_slot.1 {
                closest_slot = (slot, dist);
            }
        }
        closest_slot
    }

    pub fn get_entity_slot_position(entity: Entity, world: &World) -> Result<Vec2> {
        let context = Context::from_owner(entity, world);
        let slot = context.get_var(VarName::Slot, world).unwrap().get_int()? as usize;
        let faction = context
            .get_var(VarName::Faction, world)
            .unwrap()
            .get_faction()?;
        Ok(Self::get_slot_position(faction, slot))
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
        if let Some(team) = PackedTeam::find_entity(faction, world) {
            if let Some(children) = world.get::<Children>(team) {
                return children
                    .iter()
                    .filter_map(|e| world.get::<Unit>(*e).map(|_| *e))
                    .collect_vec();
            }
        }
        default()
    }

    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        Self::make_slot_gap(faction, i32::MAX, world);
    }

    pub fn make_slot_gap(faction: Faction, slot: i32, world: &mut World) {
        let dragged = world.resource::<DraggedUnit>().0;
        for (unit, ind) in Self::collect_faction(faction, world)
            .into_iter()
            .filter(|u| dragged != Some(*u))
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
        let faction = context
            .get_var(VarName::Faction, world)
            .context("No faction var")?
            .get_faction()?;
        let slot = context
            .get_var(VarName::Slot, world)
            .context("No slot var")?
            .get_int()?;
        let pos = Self::get_slot_position(faction, slot as usize);
        VarState::push_back(
            entity,
            VarName::Position,
            VarChange::new(VarValue::Vec2(pos)),
            world,
        );
        Ok(())
    }

    pub fn translate_unit(entity: Entity, position: Vec2, world: &mut World) {
        ActionCluster::current(world).push_change(
            entity,
            ChangeType::Var {
                var: VarName::Position,
                change: VarChange::new(VarValue::Vec2(position))
                    .set_duration(0.3)
                    .set_tween(Tween::QuartOut),
            },
        );
    }

    pub fn run_death_check(already_died: &HashSet<Entity>, world: &mut World) -> Vec<Entity> {
        let dead = world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .filter(|e| !already_died.contains(e) && Self::is_dead(*e, world))
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
        let mut unit = world.entity_mut(entity);
        unit.remove::<Unit>();
        unit.insert(Corpse);
        VarState::push_back(
            unit.id(),
            VarName::Visible,
            VarChange::new(VarValue::Bool(false)),
            world,
        );
    }

    pub fn is_dead(entity: Entity, world: &World) -> bool {
        Context::from_owner(entity, world)
            .get_var(VarName::Hp, world)
            .unwrap()
            .get_int()
            .unwrap()
            <= 0
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
            .query_filtered::<Entity, Or<(&Unit, &Corpse)>>()
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
        for (unit, faction) in units.into_iter() {
            let slot = VarState::get(unit, world).get_int(VarName::Slot).unwrap() as usize;
            UnitPlugin::translate_unit(unit, UnitPlugin::get_slot_position(faction, slot), world);
        }
    }

    pub fn hover_unit(event: Listener<Pointer<Over>>, mut hovered: ResMut<HoveredUnit>) {
        hovered.0 = Some(event.target);
    }

    pub fn unhover_unit(_event: Listener<Pointer<Out>>, mut hovered: ResMut<HoveredUnit>) {
        hovered.0 = None;
    }

    pub fn drag_unit_start(
        event: Listener<Pointer<DragStart>>,
        mut team: Query<Entity, With<ActiveTeam>>,
        mut dragged: ResMut<DraggedUnit>,
        mut commands: Commands,
    ) {
        debug!("Drag unit start {:?}", event.target);
        dragged.0 = Some(event.target);
        for entity in team.iter_mut() {
            commands.entity(entity).insert(Pickable::IGNORE);
        }
        commands.insert_resource(ClosestSlot(0, f32::MAX, false));
    }

    pub fn change_stacks(unit: Entity, delta: i32, world: &mut World) {
        VarState::change_int(unit, VarName::Stacks, delta, world).unwrap();
        VarState::change_int(unit, VarName::Hp, delta, world).unwrap();
        VarState::change_int(unit, VarName::Atk, delta, world).unwrap();
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
            let dragged = world.resource::<DraggedUnit>().0.unwrap();
            world.resource_mut::<DraggedUnit>().0 = None;
            let ClosestSlot(slot, _, stackable) = *world.resource::<ClosestSlot>();
            let target = Self::find_unit(Faction::Team, slot, world);
            if stackable && !target.is_some_and(|target| target.eq(&dragged)) {
                world.entity_mut(dragged).despawn_recursive();
                Self::change_stacks(target.unwrap(), 1, world);
            } else {
                let t = GameTimer::get(world).insert_head();
                VarState::get_mut(dragged, world).insert_simple(
                    VarName::Slot,
                    VarValue::Int(slot as i32),
                    t,
                );
            }
            Self::fill_slot_gaps(Faction::Team, world);
            Self::translate_to_slots(world);
        });
    }

    pub fn drag_unit(world: &mut World) {
        if let Some(dragged) = world.resource::<DraggedUnit>().0 {
            if let Some(cursor_pos) = CameraPlugin::cursor_world_pos(world) {
                let mut transform = world.get_mut::<Transform>(dragged).unwrap();
                transform.translation = cursor_pos.extend(0.0);
                VarState::get_mut(dragged, world)
                    .init(VarName::Position, VarValue::Vec2(cursor_pos));
                if let Some(prev_closest) = world.get_resource::<ClosestSlot>() {
                    let closest_slot = Self::get_closest_slot(cursor_pos, Faction::Team);
                    if prev_closest.0 != closest_slot.0 {
                        let name = VarState::get(dragged, world)
                            .get_string(VarName::Name)
                            .unwrap();
                        let stackable = if Self::find_unit(Faction::Team, closest_slot.0, world)
                            .is_some_and(|entity| {
                                VarState::get(entity, world)
                                    .get_string(VarName::Name)
                                    .unwrap()
                                    .eq(&name)
                            }) {
                            true
                        } else {
                            Self::make_slot_gap(Faction::Team, closest_slot.0 as i32, world);
                            Self::translate_to_slots(world);
                            false
                        };
                        world.insert_resource(ClosestSlot(
                            closest_slot.0,
                            closest_slot.1,
                            stackable,
                        ));
                    }
                }
            }
        }
    }

    fn ui(world: &mut World) {
        let hovered = world.get_resource::<HoveredUnit>().unwrap().0;
        let ctx = &egui_context(world);
        for (entity, card) in world.query::<(Entity, &UnitCard)>().iter(world) {
            card.show_window(card.open || hovered == Some(entity), ctx, world);
        }
    }

    pub fn get_faction(unit: Entity, world: &World) -> Faction {
        Context::from_owner(unit, world)
            .get_var(VarName::Faction, world)
            .unwrap()
            .get_faction()
            .unwrap()
    }

    pub fn spawn_slot(slot: usize, faction: Faction, world: &mut World) -> Entity {
        let pos = UnitPlugin::get_slot_position(Faction::Team, slot);
        let team = PackedTeam::find_entity(faction, world);
        let rep = Options::get_slot_rep(world)
            .clone()
            .unpack(None, team, world);
        let mut state = VarState::new_with(VarName::Position, VarValue::Vec2(pos));
        state.init(VarName::Slot, VarValue::Int(slot as i32));
        state.attach(rep, world);
        world.get_mut::<Transform>(rep).unwrap().translation.z -= 100.0;
        world.entity_mut(rep).insert(Slot);
        rep
    }
}

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct UnitRepresentation;

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
        PackedTeam::find_entity(*self, world).unwrap_or_else(|| PackedTeam::spawn(*self, world))
    }
}

#[derive(Resource, Default)]
pub struct HoveredUnit(pub Option<Entity>);

#[derive(Resource, Default, Debug)]
pub struct DraggedUnit(pub Option<Entity>);

#[derive(Component)]
pub struct ActiveTeam;

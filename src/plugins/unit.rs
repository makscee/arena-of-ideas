use bevy_egui::egui::{CollapsingHeader, RichText};
use strum_macros::Display;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredUnit>()
            .init_resource::<DraggedUnit>()
            .add_systems(Update, Self::ui);
    }
}

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
            Faction::Team => vec2(slot as f32 * -3.0 + 7.5, -3.0),
            Faction::Shop => vec2(slot as f32 * -3.0 + 7.5, 3.0),
        }
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
            .map(|f| {
                Self::collect_faction(f, world)
                    .into_iter()
                    .map(move |e| (e, f))
            })
            .flatten()
            .collect_vec()
    }

    pub fn collect_faction(faction: Faction, world: &mut World) -> Vec<Entity> {
        if let Some(team) = PackedTeam::entity(faction, world) {
            if let Some(children) = world.get::<Children>(team) {
                return children
                    .iter()
                    .filter_map(|e| match world.get::<Unit>(*e) {
                        Some(_) => Some(*e),
                        None => None,
                    })
                    .collect_vec();
            }
        }
        return default();
    }

    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        for (slot, unit) in Self::collect_faction(faction, world)
            .into_iter()
            .sorted_by_key(|x| VarState::get(*x, world).get_int(VarName::Slot).unwrap())
            .enumerate()
            .collect_vec()
        {
            VarState::get_mut(unit, world).init(VarName::Slot, VarValue::Int(slot as i32 + 1));
        }
    }

    pub fn clear_world(world: &mut World) {
        let entities = world
            .query_filtered::<Entity, Or<(&Unit, &Corpse)>>()
            .iter(world)
            .collect_vec();
        for entity in entities {
            world.entity_mut(entity).despawn_recursive();
        }
    }

    pub fn translate_unit(entity: Entity, position: Vec2, world: &mut World) {
        VarState::push_back(
            entity,
            VarName::Position,
            Change::new(VarValue::Vec2(position))
                .set_duration(0.3)
                .set_tween(Tween::QuartOut),
            world,
        );
    }

    /// Iter over all Units
    /// For any hp < 0 replace Unit marker with Corpse marker
    pub fn run_death_check(world: &mut World) {
        let dead = world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .filter(|e| Self::is_dead(*e, world))
            .collect_vec();
        for unit in dead {
            Self::turn_into_corpse(unit, world);
        }
    }

    pub fn turn_into_corpse(entity: Entity, world: &mut World) {
        let mut unit = world.entity_mut(entity);
        unit.remove::<Unit>();
        unit.insert(Corpse);
        VarState::push_back(
            unit.id(),
            VarName::Visible,
            Change::new(VarValue::Bool(false)),
            world,
        );
        Event::Death(entity).send(world);
        let killer = VarState::get(entity, world)
            .get_entity(VarName::LastAttacker)
            .unwrap();
        Event::Kill {
            owner: killer,
            target: entity,
        }
        .send(world);
    }

    pub fn is_dead(entity: Entity, world: &World) -> bool {
        Context::from_owner(entity, world)
            .get_var(VarName::Hp, world)
            .unwrap()
            .get_int()
            .unwrap()
            <= 0
    }

    pub fn despawn_all(world: &mut World) {
        for team in world
            .query_filtered::<Entity, With<Team>>()
            .iter(world)
            .collect_vec()
        {
            debug!("Despawn {team:?}");
            world.entity_mut(team).despawn_recursive()
        }
        Representation::despawn_all(world);
    }

    pub fn translate_to_slots(world: &mut World) {
        let units = UnitPlugin::collect_factions(
            HashSet::from([Faction::Left, Faction::Right, Faction::Team, Faction::Shop]),
            world,
        );
        GameTimer::get_mut(world).start_batch();
        for (unit, faction) in units.into_iter() {
            let slot = VarState::get(unit, world).get_int(VarName::Slot).unwrap() as usize;
            GameTimer::get_mut(world).head_to_batch_start();
            UnitPlugin::translate_unit(unit, UnitPlugin::get_slot_position(faction, slot), world);
        }
        GameTimer::get_mut(world).end_batch();
    }

    pub fn hover_unit(event: Listener<Pointer<Over>>, mut hovered: ResMut<HoveredUnit>) {
        debug!("Hover unit start {:?}", event.target);
        hovered.0 = Some(event.target);
    }

    pub fn unhover_unit(event: Listener<Pointer<Out>>, mut hovered: ResMut<HoveredUnit>) {
        debug!("Hover unit end {:?}", event.target);
        hovered.0 = None;
    }

    pub fn drag_unit_start(
        event: Listener<Pointer<DragStart>>,
        mut dragged: ResMut<DraggedUnit>,
        mut commands: Commands,
    ) {
        debug!("Drag unit start {:?}", event.target);
        dragged.0 = Some(event.target);
        commands.entity(event.target).insert(Pickable::IGNORE);
    }

    pub fn drag_unit_end(
        event: Listener<Pointer<DragEnd>>,
        mut dragged: ResMut<DraggedUnit>,
        mut commands: Commands,
    ) {
        debug!("Drag unit end {:?}", event.target);
        dragged.0 = None;
        commands.entity(event.target).insert(Pickable::default());
    }

    pub fn drag_unit(
        event: Listener<Pointer<Drag>>,
        mut query_transform: Query<&mut Transform>,
        query_camera: Query<(&Camera, &GlobalTransform)>,
    ) {
        let entity = event.target;
        if let Ok(mut transform) = query_transform.get_mut(entity) {
            let (camera, camera_transform) = query_camera.single();
            let delta = screen_to_world(event.delta, camera, camera_transform)
                - screen_to_world(Vec2::ZERO, camera, camera_transform);
            transform.translation += delta.extend(0.0);
        }
    }

    pub fn drop_unit(
        event: Listener<Pointer<Drop>>,
        mut unit_query: Query<&mut VarState, (With<Unit>, Without<Team>, Without<Slot>)>,
        slot_query: Query<&VarState, With<Slot>>,
        team_query: Query<(&VarState, &Children), With<Team>>,
        timer: Res<GameTimer>,
    ) {
        let unit = event.dropped;
        dbg!(&event);
        if !unit_query.contains(unit) {
            debug!("Non unit dropped {unit:?}");
            return;
        }
        let (slot, position) = {
            let slot = slot_query.get(event.target).unwrap();
            (
                slot.get_int(VarName::Slot).unwrap(),
                slot.get_vec2(VarName::Position).unwrap(),
            )
        };
        let team_units = team_query
            .iter()
            .find(|(state, _)| {
                state
                    .get_faction(VarName::Faction)
                    .unwrap()
                    .eq(&Faction::Team)
            })
            .unwrap()
            .1
            .to_vec();
        for unit in team_units {
            if let Ok(state) = unit_query.get(unit) {
                if slot == state.get_int(VarName::Slot).unwrap() {
                    debug!("Slot occupied {slot} {unit:?}");
                    return;
                }
            }
        }
        let t = timer.get_insert_t();
        unit_query
            .get_mut(unit)
            .unwrap()
            .insert_simple(VarName::Slot, VarValue::Int(slot), t)
            .insert_simple(VarName::Position, VarValue::Vec2(position), t);
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        if let Some(hovered) = world.get_resource::<HoveredUnit>().unwrap().0 {
            if world.get::<ShopOffer>(hovered).is_some() {
                return;
            }
            let description = VarState::get(hovered, world)
                .get_string(VarName::Description)
                .unwrap_or_default();
            if !description.is_empty() {
                entity_panel(hovered, vec2(0.0, 1.0), "hover_desc", world).show(ctx, |ui| {
                    ui.label(description);
                });
            }
            let statuses = Status::collect_entity_statuses(hovered, world);
            if !statuses.is_empty() {
                entity_panel(hovered, vec2(1.0, 0.0), "Statuses", world)
                    .title_bar(true)
                    .show(ctx, |ui| {
                        ui.set_width(150.0);
                        ui.vertical_centered(|ui| {
                            for status in statuses {
                                let state = VarState::get(status, world);
                                let t = get_t(world);
                                if state.birth > t {
                                    continue;
                                }
                                if let Ok(name) = state.get_string(VarName::Name) {
                                    let description =
                                        state.get_string(VarName::Description).unwrap();
                                    let charges =
                                        VarState::get_value(status, VarName::Charges, t, world)
                                            .unwrap()
                                            .get_int()
                                            .unwrap();
                                    let name = format!("{name} ({charges})");
                                    CollapsingHeader::new(
                                        RichText::new(name).color(hex_color!("#2196F3")),
                                    )
                                    .default_open(true)
                                    .show(ui, |ui| ui.label(description));
                                }
                            }
                        })
                    });
            }
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
        let team = PackedTeam::entity(faction, world);
        let rep = Options::get_slot_rep(world)
            .clone()
            .unpack(None, team, world);
        let mut state = VarState::new_with(VarName::Position, VarValue::Vec2(pos));
        state.init(VarName::Slot, VarValue::Int(slot as i32));
        state.attach(rep, world);
        world.get_mut::<Transform>(rep).unwrap().translation.z -= 100.0;
        world
            .entity_mut(rep)
            .insert(Slot)
            .insert((RaycastPickTarget::default(), PickableBundle::default()))
            .insert(On::<Pointer<Drop>>::run(Self::drop_unit));
        rep
    }
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

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
)]
pub enum Faction {
    Left,
    Right,
    Team,
    Shop,
}

#[derive(Resource, Default)]
pub struct HoveredUnit(pub Option<Entity>);

#[derive(Resource, Default, Debug)]
pub struct DraggedUnit(pub Option<Entity>);

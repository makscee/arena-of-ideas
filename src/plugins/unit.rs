use std::collections::VecDeque;

use super::*;

pub const SACRIFICE_SLOT: usize = 6;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredUnit>()
            .init_resource::<DraggedUnit>()
            .add_systems(Update, Self::ui);
    }
}

#[derive(Resource, Debug)]
pub struct ClosestSlot(usize, f32);

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
            Faction::Team => vec2(slot as f32 * -3.0 + 12.5, -3.0),
            Faction::Shop => vec2(slot as f32 * -3.0 + 6.5, 1.0),
        }
    }

    pub fn get_closest_slot(pos: Vec2, faction: Faction) -> (usize, f32) {
        let mut closest_slot = (0, f32::MAX);
        for slot in 0..10 {
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

    /// Iter over all Units
    /// For any hp < 0 replace Unit marker with Corpse marker
    pub fn run_death_check(world: &mut World) -> bool {
        let dead = world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .filter(|e| Self::is_dead(*e, world))
            .collect_vec();
        let has_dead = !dead.is_empty();
        for unit in dead {
            Self::turn_into_corpse(unit, world);
        }
        has_dead
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
        Event::Death(entity).send(world);
        if let Ok(killer) = VarState::get(entity, world).get_entity(VarName::LastAttacker) {
            Event::Kill {
                owner: killer,
                target: entity,
            }
            .send(world);
        }
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
        commands.insert_resource(ClosestSlot(0, f32::MAX));
    }

    pub fn drag_unit_end(
        event: Listener<Pointer<DragEnd>>,
        mut dragged: ResMut<DraggedUnit>,
        closest_slot: Res<ClosestSlot>,
        timer: Res<GameTimer>,
        mut team: Query<Entity, With<ActiveTeam>>,
        mut state: Query<&mut VarState>,
        mut commands: Commands,
    ) {
        debug!("Drag unit end {:?}", event.target);
        let unit = event.target;
        dragged.0 = None;
        for entity in team.iter_mut() {
            commands.entity(entity).insert(Pickable::default());
        }
        let mut state = state.get_mut(unit).unwrap();
        state.insert_simple(
            VarName::Slot,
            VarValue::Int(closest_slot.0 as i32),
            timer.insert_head(),
        );
        commands.add(|world: &mut World| {
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
                        Self::make_slot_gap(Faction::Team, closest_slot.0 as i32, world);
                        Self::translate_to_slots(world);
                        world.insert_resource(ClosestSlot(closest_slot.0, closest_slot.1));
                    }
                }
            }
        }
    }

    fn ui(world: &mut World) {
        if let Some(hovered) = world.get_resource::<HoveredUnit>().unwrap().0 {
            if let Ok(card) = UnitCard::from_entity(hovered, world) {
                card.show(world);
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

pub struct UnitCard {
    pub name: ColoredString,
    pub hp: i32,
    pub atk: i32,
    pub description: ColoredString,
    pub definitions: Vec<(ColoredString, ColoredString)>,
    pub statuses: Vec<(ColoredString, i32, ColoredString)>,
    pub pos: Vec2,
    pub entity: Entity,
}

impl UnitCard {
    pub fn from_entity(entity: Entity, world: &mut World) -> Result<Self> {
        let t = get_play_head(world);
        let statuses = Status::collect_entity_statuses(entity, world)
            .into_iter()
            .filter_map(|e| {
                let state = VarState::get(e, world);
                let name = state.get_string_at(VarName::Name, t);
                if let Ok(name) = name {
                    let color: Color32 = Pools::get_status_house(&name, world)
                        .unwrap()
                        .color
                        .clone()
                        .into();
                    let description =
                        if let Some(status) = Pools::get_status(&name.to_string(), world) {
                            status.description.clone().to_colored().inject_vars(state)
                        } else {
                            ColoredString::default()
                        };
                    Some((
                        state
                            .get_string_at(VarName::Name, t)
                            .unwrap()
                            .add_color(color),
                        state.get_int_at(VarName::Charges, t).unwrap(),
                        description,
                    ))
                } else {
                    None
                }
            })
            .collect_vec();
        let pos = entity_screen_pos(entity, world);
        let state = VarState::get(entity, world);
        let description = state.get_string_at(VarName::Description, t)?;
        let mut definitions: Vec<(ColoredString, ColoredString)> = default();
        let mut added_definitions: HashSet<String> = default();
        let mut raw_definitions = VecDeque::from_iter(description.extract_bracketed(("[", "]")));
        while let Some(name) = raw_definitions.pop_front() {
            let (color, description) = if let Some(ability) = Pools::get_ability(&name, world) {
                let color: Color32 = Pools::get_ability_house(&name, world)
                    .unwrap()
                    .color
                    .clone()
                    .into();
                (color, ability.description.clone())
            } else if let Some(status) = Pools::get_status(&name, world) {
                let color: Color32 = Pools::get_status_house(&name, world)
                    .unwrap()
                    .color
                    .clone()
                    .into();
                (color, status.description.clone())
            } else {
                continue;
            };
            if !added_definitions.insert(name.clone()) {
                continue;
            }
            definitions.push((
                name.add_color(color),
                description
                    .to_colored()
                    .inject_definitions(world)
                    .inject_vars(&default()),
            ));
            raw_definitions.extend(description.extract_bracketed(("[", "]")));
        }

        let description = description
            .to_colored()
            .inject_vars(state)
            .inject_definitions(world);

        Ok(Self {
            name: state
                .get_string_at(VarName::Name, t)?
                .add_color(state.get_color_at(VarName::HouseColor, t)?.c32()),
            hp: state.get_int_at(VarName::Hp, t)?,
            atk: state.get_int_at(VarName::Atk, t)?,
            description,
            statuses,
            pos,
            entity,
            definitions,
        })
    }

    pub fn show(&self, world: &mut World) {
        self.show_with(|_| {}, world);
    }

    pub fn show_with(&self, add_contents: impl FnOnce(&mut Ui), world: &mut World) {
        window("UNIT")
            .id(self.entity)
            .set_width(200.0)
            .entity_anchor(self.entity, Align2::CENTER_BOTTOM, vec2(0.0, -60.0), world)
            .show(&egui_context(world), |ui| {
                frame(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.heading(self.name.rich_text());
                    });
                    if !self.description.is_empty() {
                        ui.vertical(|ui| {
                            ui.label(self.description.widget());
                        });
                    }
                });
                for (name, text) in &self.definitions {
                    frame(ui, |ui| {
                        ui.label(name.widget());
                        ui.horizontal_wrapped(|ui| {
                            ui.label(text.widget());
                        });
                    });
                }
                if !self.statuses.is_empty() {
                    frame(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(RichText::new("Statuses").color(white()));
                        });
                        for (name, charges, description) in self.statuses.iter() {
                            text_dots_text(name, &charges.to_string().add_color(white()), ui);
                            if !description.is_empty() {
                                ui.vertical(|ui| {
                                    ui.label(description.widget());
                                });
                            }
                        }
                    });
                }
                add_contents(ui);
            });
    }
}

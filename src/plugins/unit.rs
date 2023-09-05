use bevy::utils::Instant;
use bevy_egui::egui::{CollapsingHeader, RichText};
use strum_macros::Display;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredUnit>()
            .add_systems(OnEnter(GameState::Restart), Self::despawn)
            .add_systems(Update, Self::test_system)
            .add_systems(Update, Self::ui);
    }
}

impl UnitPlugin {
    fn test_system(world: &mut World) {
        if !world
            .get_resource::<Input<KeyCode>>()
            .unwrap()
            .just_pressed(KeyCode::T)
        {
            return;
        }
        let units = world
            .query_filtered::<Entity, With<Unit>>()
            .iter(world)
            .collect_vec();
        for unit in units {
            debug!("Change charges of {unit:?}");
            Status::change_charges("Test", unit, 1, world).unwrap();
        }
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
        let state = VarState::get(entity, world);
        let slot = state.get_int(VarName::Slot)? as usize;
        let faction = state.get_faction(VarName::Faction)?;
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
                return children.iter().cloned().collect_vec();
            }
        }
        return default();
    }

    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        let team = PackedTeam::entity(faction, world).unwrap();
        for (slot, unit) in world
            .get::<Children>(team)
            .unwrap()
            .iter()
            .sorted_by_key(|x| VarState::get(**x, world).get_int(VarName::Slot).unwrap())
            .cloned()
            .enumerate()
            .collect_vec()
        {
            VarState::get_mut(unit, world).insert(VarName::Slot, VarValue::Int(slot as i32 + 1));
        }
    }

    fn despawn(
        query: Query<Entity, Or<(&Unit, &Corpse, &ShopOffer)>>,
        mut commands: Commands,
        mut state: ResMut<NextState<GameState>>,
        mut time: ResMut<Time>,
        mut game_timer: ResMut<GameTimer>,
    ) {
        for unit in query.iter() {
            commands.entity(unit).despawn_recursive();
        }
        state.set(GameState::Shop);
        *time = Time::new(Instant::now());
        game_timer.reset();
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
    }

    pub fn is_dead(entity: Entity, world: &World) -> bool {
        world
            .get::<VarState>(entity)
            .unwrap()
            .get_int(VarName::Hp)
            .unwrap()
            <= 0
    }

    pub fn despawn_all(world: &mut World) {
        for unit in world
            .query_filtered::<Entity, Or<(&Unit, &Corpse)>>()
            .iter(world)
            .collect_vec()
        {
            debug!("Despawn {unit:?}");
            world.entity_mut(unit).despawn_recursive()
        }
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
            let statuses = Status::collect_all_statuses(hovered, world);
            if !statuses.is_empty() {
                entity_panel(hovered, vec2(1.0, 0.0), "Statuses", world)
                    .title_bar(true)
                    .show(ctx, |ui| {
                        ui.set_width(150.0);
                        ui.vertical_centered(|ui| {
                            for status in statuses {
                                let state = VarState::get(status, world);
                                let name = state.get_string(VarName::Name).unwrap();
                                let description = state.get_string(VarName::Description).unwrap();
                                let charges = state.get_int(VarName::Charges).unwrap_or(1);
                                let name = format!("{name} ({charges})");
                                CollapsingHeader::new(
                                    RichText::new(name).color(hex_color!("#2196F3")),
                                )
                                .default_open(true)
                                .show(ui, |ui| ui.label(description));
                            }
                        })
                    });
            }
        }
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
pub struct HoveredUnit(Option<Entity>);

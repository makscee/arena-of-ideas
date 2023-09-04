use bevy::utils::Instant;
use bevy_egui::{
    egui::{self, Align2, Id, Pos2, Ui},
    EguiContext,
};

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredUnit>()
            .add_systems(OnEnter(GameState::Next), Self::spawn)
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

    pub fn collect_faction(faction: Faction, world: &mut World) -> Vec<Entity> {
        Self::collect_factions(&HashSet::from([faction]), world)
            .into_iter()
            .map(|x| x.0)
            .collect_vec()
    }

    pub fn collect_factions(
        factions: &HashSet<Faction>,
        world: &mut World,
    ) -> Vec<(Entity, Faction)> {
        world
            .query_filtered::<(Entity, &VarState), With<Unit>>()
            .iter(world)
            .filter_map(|(e, state)| {
                let faction = state.get_faction(VarName::Faction).unwrap();
                if factions.contains(&faction) {
                    Some((e, faction))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    pub fn fill_slot_gaps(faction: Faction, world: &mut World) {
        for (slot, mut s) in world
            .query_filtered::<&mut VarState, With<Unit>>()
            .iter_mut(world)
            .filter(|s| s.get_faction(VarName::Faction).unwrap() == faction)
            .sorted_by_key(|s| s.get_int(VarName::Slot).unwrap())
            .enumerate()
        {
            s.insert(VarName::Slot, VarValue::Int(slot as i32 + 1));
        }
    }

    fn spawn(world: &mut World) {
        for (i, (_, unit)) in world
            .get_resource::<Pools>()
            .unwrap()
            .heroes
            .clone()
            .into_iter()
            .enumerate()
        {
            let units = world.get_resource::<Assets<PackedUnit>>().unwrap();
            let unit = units.get(&unit).unwrap().clone();
            unit.unpack(Faction::Left, Some(i + 1), world);
        }
        dbg!(Options::get_custom_battle(world));
    }

    fn despawn(
        query: Query<Entity, Or<(&Unit, &Corpse)>>,
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
            &HashSet::from([Faction::Left, Faction::Right, Faction::Team, Faction::Shop]),
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
        if let Some(hovered) = world.get_resource::<HoveredUnit>().unwrap().0 {
            let description = VarState::get(hovered, world)
                .get_string(VarName::Description)
                .unwrap_or("No description".to_owned());
            Self::draw_unit_panel(hovered, vec2(0.0, 1.0), world).show(
                &egui_context(world),
                |ui| {
                    ui.label(description);
                },
            );
        }
    }

    pub fn draw_unit_panel(entity: Entity, side: Vec2, world: &mut World) -> egui::Window<'static> {
        let pos = Self::unit_screen_pos(entity, side, world);
        let side_i = side.as_ivec2();
        let align = match (side_i.x, side_i.y) {
            (-1, 0) => Align2::RIGHT_CENTER,
            (1, 0) => Align2::LEFT_CENTER,
            (0, -1) => Align2::CENTER_TOP,
            (0, 1) => Align2::CENTER_BOTTOM,
            _ => panic!(),
        };
        egui::Window::new("Hero")
            .id(Id::new(entity).with(side.x as i32).with(side.y as i32))
            .fixed_pos(Pos2::new(pos.x, pos.y))
            .default_width(10.0)
            .pivot(align)
    }

    fn unit_screen_pos(entity: Entity, offset: Vec2, world: &mut World) -> Vec2 {
        let (camera, transform) = world.query::<(&Camera, &GlobalTransform)>().single(world);
        let pos = world.get::<GlobalTransform>(entity).unwrap().translation()
            + vec3(offset.x, offset.y, 0.0);
        camera.world_to_viewport(transform, pos).unwrap()
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
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, PartialOrd, Ord,
)]
pub enum Faction {
    Left,
    Right,
    Team,
    Shop,
}

#[derive(Resource, Default)]
pub struct HoveredUnit(Option<Entity>);

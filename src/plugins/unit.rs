use bevy::utils::Instant;

use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Next), Self::spawn)
            .add_systems(OnEnter(GameState::Restart), Self::despawn);
    }
}

impl UnitPlugin {
    pub fn get_slot_position(faction: Faction, slot: usize) -> Vec2 {
        vec2(
            slot as f32
                * 3.0
                * match faction {
                    Faction::Left => -1.0,
                    Faction::Right => 1.0,
                },
            0.0,
        )
    }

    pub fn get_entity_slot_position(entity: Entity, world: &World) -> Result<Vec2> {
        let state = VarState::get(entity, world);
        let slot = state.get_int(VarName::Slot)? as usize;
        let faction = state.get_faction(VarName::Faction)?;
        Ok(Self::get_slot_position(faction, slot))
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
        state.set(GameState::Battle);
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
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Corpse;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, PartialOrd, Ord,
)]
pub enum Faction {
    Left,
    Right,
}

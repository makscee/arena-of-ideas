use bevy_egui::egui::{Align2, Vec2, Window};

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::enter)
            .add_systems(OnExit(GameState::Battle), Self::leave)
            .add_systems(Update, Self::ui.run_if(in_state(GameState::Battle)));
    }
}

#[derive(Resource, Clone)]
pub struct BattleData {
    pub left: Option<PackedTeam>,
    pub right: Option<PackedTeam>,
    pub result: BattleResult,
}

impl BattlePlugin {
    pub fn enter(world: &mut World) {
        GameTimer::get_mut(world).reset();
        let data = world.resource::<BattleData>().clone();
        data.left.unwrap().unpack(Faction::Left, world);
        data.right.unwrap().unpack(Faction::Right, world);
        UnitPlugin::translate_to_slots(world);
        Self::run_battle(100, world);
        world
            .get_resource_mut::<GameTimer>()
            .unwrap()
            .head_to_save();
    }

    pub fn load_teams(left: PackedTeam, right: PackedTeam, world: &mut World) {
        world.insert_resource(BattleData {
            left: Some(left),
            right: Some(right),
            result: default(),
        });
    }

    pub fn leave(world: &mut World) {
        UnitPlugin::despawn_all(world);
    }

    pub fn run_battle(bpm: usize, world: &mut World) -> BattleResult {
        let btime = 60.0 / bpm as f32;
        Event::BattleStart.send(world);
        ActionPlugin::spin(world);
        while let Some((left, right)) = Self::get_strikers(world) {
            Self::run_strike(left, right, btime, world);
        }
        ActionPlugin::spin(world);
        Self::get_result(world)
    }

    fn get_result(world: &mut World) -> BattleResult {
        let mut result: HashMap<Faction, usize> = default();
        for unit in world.query_filtered::<Entity, With<Unit>>().iter(world) {
            let team = get_parent(unit, world);
            let faction = VarState::get(team, world)
                .get_faction(VarName::Faction)
                .unwrap();
            *result.entry(faction).or_default() += 1;
        }
        match result.len() {
            0 => BattleResult::Even,
            1 => {
                let (faction, count) = result.iter().exactly_one().unwrap();
                match faction {
                    Faction::Left => BattleResult::Left(*count),
                    Faction::Right => BattleResult::Right(*count),
                    _ => panic!("Non-battle winning faction"),
                }
            }
            _ => panic!("Non-unique winning faction"),
        }
    }

    pub fn get_strikers(world: &mut World) -> Option<(Entity, Entity)> {
        if let Some(left) = UnitPlugin::find_unit(Faction::Left, 1, world) {
            if let Some(right) = UnitPlugin::find_unit(Faction::Right, 1, world) {
                return Some((left, right));
            }
        }
        None
    }

    pub fn run_strike(left: Entity, right: Entity, btime: f32, world: &mut World) {
        GameTimer::get_mut(world).start_batch();
        UnitPlugin::translate_to_slots(world);
        Self::before_strike(left, right, btime, world);
        GameTimer::get_mut(world).advance_end(btime).end_batch();

        GameTimer::get_mut(world).start_batch();
        Self::strike(left, right, world);
        Self::after_strike(left, right, world);
        UnitPlugin::run_death_check(world);
        UnitPlugin::fill_slot_gaps(Faction::Left, world);
        UnitPlugin::fill_slot_gaps(Faction::Right, world);
        ActionPlugin::spin(world);
        GameTimer::get_mut(world).advance_end(btime).end_batch();
    }

    fn before_strike(left: Entity, right: Entity, btime: f32, world: &mut World) {
        GameTimer::get_mut(world).head_to_batch_start();
        Event::TurnStart.send(world);
        Event::BeforeStrike(left).send(world);
        Event::BeforeStrike(right).send(world);
        if ActionPlugin::spin(world) {
            GameTimer::get_mut(world)
                .head_to_batch_start()
                .advance_end(btime)
                .end_batch()
                .start_batch();
        }
        let units = vec![(left, -1.0), (right, 1.0)];
        for (caster, dir) in units {
            GameTimer::get_mut(world).head_to_batch_start();
            Options::get_animations(world)
                .get(AnimationType::BeforeStrike)
                .clone()
                .apply(
                    &Context::from_owner(caster, world)
                        .set_var(VarName::Direction, VarValue::Float(dir)),
                    world,
                )
                .unwrap();
        }
    }

    fn strike(left: Entity, right: Entity, world: &mut World) {
        debug!("Strike {left:?} {right:?}");
        let units = vec![(left, right), (right, left)];
        for (caster, target) in units {
            GameTimer::get_mut(world).head_to_batch_start();
            let context = mem::take(
                Context::from_caster(caster, world)
                    .set_target(target, world)
                    .set_owner(caster, world),
            );
            let effect = Effect::Damage(None);
            ActionPlugin::push_back(effect, context, world);
            ActionPlugin::spin(world);
        }
    }

    fn after_strike(left: Entity, right: Entity, world: &mut World) {
        debug!("After strike {left:?} {right:?}");
        let units = vec![left, right];
        for caster in units {
            GameTimer::get_mut(world).head_to_batch_start();
            Options::get_animations(world)
                .get(AnimationType::AfterStrike)
                .clone()
                .apply(&Context::from_owner(caster, world), world)
                .unwrap();
        }
    }

    pub fn ui(world: &mut World) {
        Window::new("")
            .title_bar(false)
            .show(&egui_context(world), |ui| {
                if ui.button("Skip").clicked() {
                    let mut timer = GameTimer::get_mut(world);
                    let end = timer.end();
                    timer.set_t(end);
                }
            });
        if !GameTimer::get(world).ended() {
            return;
        }
        Window::new("Battle over")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(&egui_context(world), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Ok").clicked() {
                        GameState::change(GameState::Shop, world);
                    }
                    if ui.button("Replay").clicked() {
                        GameTimer::get_mut(world).set_t(0.0);
                    }
                })
            });
    }
}

#[derive(Debug, Default, Clone)]
pub enum BattleResult {
    #[default]
    Tbd,
    Left(usize),
    Right(usize),
    Even,
}

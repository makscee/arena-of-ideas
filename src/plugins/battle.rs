use bevy_egui::egui::Align2;

use crate::module_bindings::{run_submit_result, ArenaRun};

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), Self::on_leave)
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
    pub fn on_enter(world: &mut World) {
        GameTimer::get().reset();
        let result = Self::run_battle(world).unwrap();
        if let Some(win) = result.is_win() {
            run_submit_result(win);
        } else {
            error!("Failed to get battle result");
        }
        let mut bd = world.resource_mut::<BattleData>();
        bd.result = result;
    }

    pub fn load_teams(left: PackedTeam, right: PackedTeam, world: &mut World) {
        world.insert_resource(BattleData {
            left: Some(left),
            right: Some(right),
            result: default(),
        });
    }

    pub fn on_leave(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
    }

    pub fn run_battle(world: &mut World) -> Result<BattleResult> {
        let timeframe = AudioPlugin::beat_timeframe();
        ActionPlugin::set_timeframe(timeframe, world);
        let shift_left = -AudioPlugin::to_next_beat(world);
        GameTimer::get()
            .advance_insert(shift_left)
            .advance_play(shift_left);
        let data = world.resource::<BattleData>().clone();
        data.left.unwrap().unpack(Faction::Left, world);
        data.right.unwrap().unpack(Faction::Right, world);
        UnitPlugin::translate_to_slots(world);
        ActionPlugin::spin(world);
        GameTimer::get().insert_head_to(0.0);
        Event::BattleStart.send(world).spin(world);
        while let Some((left, right)) = Self::get_strikers(world) {
            Self::run_strike(left, right, world);
        }
        ActionPlugin::spin(world);
        Self::get_result(world)
    }

    fn get_result(world: &mut World) -> Result<BattleResult> {
        let mut result: HashMap<Faction, usize> = default();
        for unit in world.query_filtered::<Entity, With<Unit>>().iter(world) {
            let team = get_parent(unit, world);
            let faction = VarState::get(team, world)
                .get_faction(VarName::Faction)
                .unwrap();
            *result.entry(faction).or_default() += 1;
        }
        match result.len() {
            0 => Ok(BattleResult::Even),
            1 => {
                let (faction, count) = result.iter().exactly_one().unwrap();
                match faction {
                    Faction::Left => Ok(BattleResult::Left(*count)),
                    Faction::Right => Ok(BattleResult::Right(*count)),
                    _ => panic!("Non-battle winning faction"),
                }
            }
            _ => Err(anyhow!("Non-unique winning faction {result:#?}")),
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

    fn stricker_death_check(left: Entity, right: Entity, world: &mut World) -> bool {
        UnitPlugin::is_dead(left, world) || UnitPlugin::is_dead(right, world)
    }

    pub fn run_strike(left: Entity, right: Entity, world: &mut World) {
        ActionPlugin::spin(world);
        Self::before_strike(left, right, world);
        if Self::stricker_death_check(left, right, world) {
            return;
        }
        Self::strike(left, right, world);
        Self::after_strike(left, right, world);
        Event::TurnEnd.send(world).spin(world);
    }

    fn before_strike(left: Entity, right: Entity, world: &mut World) {
        debug!("Before strike {left:?} {right:?}");
        Event::TurnStart.send(world).spin(world);
        Event::BeforeStrike(left).send(world).spin(world);
        Event::BeforeStrike(right).send(world).spin(world);
        if Self::stricker_death_check(left, right, world) {
            return;
        }
        let units = vec![(left, -1.0), (right, 1.0)];
        for (caster, dir) in units {
            Options::get_animations(world)
                .get(AnimationType::BeforeStrike)
                .clone()
                .apply(
                    Context::from_owner(caster, world)
                        .set_var(VarName::Direction, VarValue::Float(dir))
                        .take(),
                    world,
                )
                .unwrap();
        }
        ActionPlugin::spin(world);
    }

    fn strike(left: Entity, right: Entity, world: &mut World) {
        debug!("Strike {left:?} {right:?}");
        let units = vec![(left, right), (right, left)];
        let mut actions: Vec<Action> = default();
        for (caster, target) in units {
            let context = Context::from_caster(caster, world)
                .set_target(target, world)
                .set_owner(caster, world)
                .take();
            let effect = Effect::Damage(None);
            actions.push(Action { effect, context });
        }
        ActionPlugin::new_cluster_many(actions, world);
    }

    fn after_strike(left: Entity, right: Entity, world: &mut World) {
        debug!("After strike {left:?} {right:?}");
        let units = vec![left, right];
        for caster in units {
            Options::get_animations(world)
                .get(AnimationType::AfterStrike)
                .clone()
                .apply(Context::from_owner(caster, world), world)
                .unwrap();
        }
        Event::AfterStrike(left).send(world).spin(world);
        Event::AfterStrike(right).send(world).spin(world);
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        if !GameTimer::get().ended() {
            return;
        }
        if let Some(win) = world.resource::<BattleData>().result.is_win() {
            let run = ArenaRun::current();
            let text = match win {
                true => "Victory".to_owned(),
                false => "Defeat".to_owned(),
            };
            let subtext = "Wins: "
                .to_colored()
                .push(format!("{}/10", run.state.wins), white())
                .push("\nLoses: ".to_owned(), light_gray())
                .push(format!("{}/3", run.state.loses), white())
                .take();
            let color = match win {
                true => {
                    hex_color!("#80D8FF")
                }
                false => hex_color!("#FF1744"),
            };

            window("BATTLE END")
                .set_width(400.0)
                .set_color(color)
                .anchor(Align2::CENTER_CENTER, [0.0, -200.0])
                .show(ctx, |ui| {
                    frame(ui, |ui| {
                        ui.heading(text.add_color(color).rich_text());
                        ui.label(subtext.widget());
                        ui.columns(2, |ui| {
                            ui[0].vertical_centered_justified(|ui| {
                                if ui.button("REPLAY").clicked() {
                                    let t = -AudioPlugin::to_next_beat(world);
                                    GameTimer::get().play_head_to(t);
                                }
                            });
                            ui[1].vertical_centered_justified(|ui| {
                                if ui.button_primary("OK").clicked() {
                                    match win {
                                        true => {
                                            GameState::Shop.change(world);
                                        }
                                        false => {
                                            GameState::MainMenu.change(world);
                                        }
                                    }
                                }
                            });
                        })
                    });
                });
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum BattleResult {
    #[default]
    Tbd,
    Left(usize),
    Right(usize),
    Even,
}

impl BattleResult {
    pub fn is_win(&self) -> Option<bool> {
        match self {
            BattleResult::Tbd => None,
            BattleResult::Left(..) | BattleResult::Even => Some(true),
            BattleResult::Right(..) => Some(false),
        }
    }
}

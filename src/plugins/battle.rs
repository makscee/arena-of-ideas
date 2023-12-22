use bevy_egui::egui::{Align2, Window};

use crate::module_bindings::beat_tower;

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
    pub level: Option<usize>,
    pub result: BattleResult,
    pub end: BattleEnd,
}

#[derive(Clone, Copy)]
pub enum BattleEnd {
    Defeat(usize, usize, bool),
    TowerBeaten(usize),
    TowerGenerate(usize),
    Victory(usize, usize),
}

impl BattlePlugin {
    pub fn on_enter(world: &mut World) {
        GameTimer::get_mut(world).reset();
        let result = Self::run_battle(world).unwrap();
        if matches!(Self::process_battle_result(result, world), Err(..)) {
            let mut bd = world.resource_mut::<BattleData>();
            bd.end = match result {
                BattleResult::Left(_) | BattleResult::Even => BattleEnd::Victory(0, 0),
                BattleResult::Right(_) => BattleEnd::Defeat(0, 0, false),
                BattleResult::Tbd => panic!("Failed to get BattleResult"),
            };
            bd.result = result;
        }
    }

    fn process_battle_result(result: BattleResult, world: &mut World) -> Result<()> {
        let save = Save::get(world)?;
        let level = save.climb.defeated + 1;
        let total = Tower::total_levels(world);
        let mut bd = world.resource_mut::<BattleData>();
        bd.end = match result {
            BattleResult::Tbd => panic!("Failed to get BattleResult"),
            BattleResult::Left(_) | BattleResult::Even => match level == total {
                true => match save.mode {
                    GameMode::NewTower => BattleEnd::TowerGenerate(level),
                    GameMode::RandomTower { .. } => BattleEnd::TowerBeaten(level),
                },
                false => BattleEnd::Victory(level, total),
            },
            BattleResult::Right(_) => {
                let finish_building = save.mode == GameMode::NewTower;
                BattleEnd::Defeat(level, total, finish_building)
            }
        };
        bd.result = result;
        Ok(())
    }

    pub fn load_teams(
        left: PackedTeam,
        right: PackedTeam,
        level: Option<usize>,
        world: &mut World,
    ) {
        world.insert_resource(BattleData {
            left: Some(left),
            right: Some(right),
            level,
            result: default(),
            end: BattleEnd::Defeat(0, 0, false),
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
        GameTimer::get_mut(world)
            .advance_insert(shift_left)
            .advance_play(shift_left);
        let data = world.resource::<BattleData>().clone();
        data.left.unwrap().unpack(Faction::Left, world);
        data.right.unwrap().unpack(Faction::Right, world);
        UnitPlugin::translate_to_slots(world);
        ActionPlugin::spin(world);
        GameTimer::get_mut(world).insert_head_to(0.0);
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
        if !GameTimer::get(world).ended() {
            return;
        }
        let end = world.resource::<BattleData>().end;
        let text = match end {
            BattleEnd::Defeat(..) => "Defeat".to_owned(),
            BattleEnd::TowerBeaten(..) | BattleEnd::Victory(..) | BattleEnd::TowerGenerate(..) => {
                "Victory".to_owned()
            }
        };
        let subtext = match end {
            BattleEnd::Defeat(level, total, finished_building) => {
                if finished_building {
                    format!("New tower is {level} levels long")
                } else {
                    format!("Reached level {level}/{total}")
                }
            }
            BattleEnd::TowerBeaten(level) => format!("Tower beaten! {level}/{level}"),
            BattleEnd::TowerGenerate(..) => "New levels will be generated.".to_string(),
            BattleEnd::Victory(level, total) => format!("{level}/{total}"),
        };
        let color = match end {
            BattleEnd::Defeat(..) => hex_color!("#FF1744"),
            BattleEnd::TowerBeaten(_) | BattleEnd::TowerGenerate(_) | BattleEnd::Victory(_, _) => {
                hex_color!("#80D8FF")
            }
        };

        window("BATTLE END")
            .set_width(400.0)
            .set_color(color)
            .anchor(Align2::CENTER_CENTER, [0.0, -200.0])
            .show(ctx, |ui| {
                frame(ui, |ui| {
                    ui.heading(text.add_color(color).rich_text());
                    ui.label(subtext.add_color(white()).rich_text());
                    ui.columns(2, |ui| {
                        ui[0].vertical_centered_justified(|ui| {
                            if ui.button("REPLAY").clicked() {
                                let t = -AudioPlugin::to_next_beat(world);
                                GameTimer::get_mut(world).play_head_to(t);
                            }
                        });
                        ui[1].vertical_centered_justified(|ui| {
                            if ui.button_primary("OK").clicked() {
                                match end {
                                    BattleEnd::Defeat(_, _, new) => {
                                        if new && LoginPlugin::is_connected() {
                                            Save::get(world).unwrap().finish_building_tower();
                                        }
                                        Save::clear(world).unwrap();
                                        GameState::MainMenu.change(world);
                                    }
                                    BattleEnd::TowerBeaten(_) => {
                                        let save = Save::get(world).unwrap();
                                        let level = RatingPlugin::generate_weakest_opponent(
                                            &save.climb.team,
                                            1,
                                            world,
                                        )[0]
                                        .to_tower_string();
                                        let team = ron::to_string(&save.climb.team).unwrap();
                                        let mut levels = save.climb.levels.clone();
                                        levels.push(level);
                                        beat_tower(save.get_tower_id().unwrap(), levels, team);
                                        Save::clear(world).unwrap();
                                        GameState::MainMenu.change(world);
                                    }
                                    BattleEnd::TowerGenerate(level) => {
                                        let generate_amount = match level {
                                            ..=3 => 3,
                                            4..=8 => 2,
                                            _ => 1,
                                        };
                                        let teams = RatingPlugin::generate_weakest_opponent(
                                            &Save::get(world).unwrap().climb.team,
                                            generate_amount,
                                            world,
                                        )
                                        .iter()
                                        .map(|l| l.to_tower_string())
                                        .collect_vec();
                                        Save::get(world)
                                            .unwrap()
                                            .register_victory()
                                            .add_tower_levels(teams)
                                            .save(world)
                                            .unwrap();
                                        GameState::Shop.change(world);
                                    }
                                    BattleEnd::Victory(_, _) => {
                                        Save::get(world)
                                            .unwrap()
                                            .register_victory()
                                            .save(world)
                                            .unwrap();
                                        GameState::Shop.change(world);
                                    }
                                }
                            }
                        });
                    })
                });
            });
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

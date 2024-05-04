use bevy_egui::egui::Align2;
use event::Event;

use crate::module_bindings::ArenaRun;

use self::module_bindings::{ArenaArchive, ArenaPool, GlobalSettings, User};

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), Self::on_leave)
            .add_systems(
                Update,
                Self::ui
                    .run_if(in_state(GameState::Battle))
                    .after(PanelsPlugin::ui),
            );
    }
}

#[derive(Asset, TypePath, Resource, Serialize, Deserialize, Clone, Default, Debug)]
pub struct BattleData {
    left: Option<PackedTeam>,
    right: Option<PackedTeam>,
    left_player_data: Option<PlayerData>,
    right_player_data: Option<PlayerData>,
    #[serde(default)]
    result: BattleResult,
    pub run_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerData {
    id: u64,
    name: String,
}

impl PlayerData {
    fn from_id(id: u64) -> Option<Self> {
        Some(Self {
            id,
            name: User::filter_by_id(id)?.name,
        })
    }
}

impl BattlePlugin {
    pub fn on_enter(world: &mut World) {
        GameTimer::get().reset();
        let result = Self::run_battle(world).unwrap();
        let mut bd = world.resource_mut::<BattleData>();
        bd.result = result;
        if bd.run_id.is_some() {
            if let Some(win) = result.is_win() {
                ServerOperation::SubmitResult(win).send(world).unwrap();
            } else {
                error!("Failed to get battle result");
            }
        }
    }

    pub fn load_from_run(run: ArenaRun, world: &mut World) {
        let left =
            PackedTeam::from_table_units(run.state.team.into_iter().map(|u| u.unit).collect());
        let right = if let Some(right) = run.battles.get(run.round as usize) {
            let right = ArenaPool::filter_by_id(right.enemy).unwrap().team;
            PackedTeam::from_table_units(right)
        } else {
            default()
        };

        let data = BattleData {
            left: Some(left),
            right: Some(right),
            left_player_data: PlayerData::from_id(run.user_id),
            right_player_data: run
                .battles
                .get(run.round as usize)
                .and_then(|e| ArenaPool::filter_by_id(e.enemy))
                .and_then(|e| PlayerData::from_id(e.owner)),
            run_id: Some(run.id),
            ..default()
        };
        world.insert_resource(data);
    }

    pub fn load_teams(left: PackedTeam, right: PackedTeam, world: &mut World) {
        let data = BattleData {
            left: Some(left),
            right: Some(right),
            ..default()
        };
        world.insert_resource(data);
    }

    pub fn on_leave(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
    }

    pub fn run_battle(world: &mut World) -> Result<BattleResult> {
        ActionPlugin::new_battle(world);
        let data = world.resource::<BattleData>().clone();
        data.left.unwrap().unpack(Faction::Left, world);
        data.right.unwrap().unpack(Faction::Right, world);
        UnitPlugin::translate_to_slots(world);
        GameTimer::get().insert_to_end();
        ActionPlugin::spin(world)?;
        Event::BattleStart.send(world);
        ActionPlugin::spin(world)?;
        loop {
            if let Some((left, right)) = Self::get_strikers(world) {
                Self::run_strike(left, right, world)?;
                continue;
            }
            if ActionPlugin::spin(world)? || ActionPlugin::clear_dead(world) {
                continue;
            }
            break;
        }
        Self::get_result(world)
    }

    fn get_result(world: &mut World) -> Result<BattleResult> {
        let mut result: HashMap<Faction, usize> = default();
        for unit in world.query_filtered::<Entity, With<Unit>>().iter(world) {
            let team = unit.get_parent(world).unwrap();
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

    fn striker_death_check(left: Entity, right: Entity, world: &mut World) -> bool {
        UnitPlugin::is_dead(left, world) || UnitPlugin::is_dead(right, world)
    }

    pub fn run_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        ActionPlugin::spin(world)?;
        ActionPlugin::register_next_round(world);
        Self::before_strike(left, right, world)?;
        if Self::striker_death_check(left, right, world) {
            return Ok(());
        }
        Self::strike(left, right, world)?;
        Self::after_strike(left, right, world)?;
        Self::fatigue(world)?;
        Event::TurnEnd.send(world);
        ActionPlugin::spin(world)?;
        ActionPlugin::spin(world)?;
        Ok(())
    }

    fn before_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("Before strike {left:?} {right:?}");
        Event::TurnStart.send(world);
        ActionPlugin::spin(world)?;
        Event::BeforeStrike(left, right).send(world);
        ActionPlugin::spin(world)?;
        Event::BeforeStrike(right, left).send(world);
        ActionPlugin::spin(world)?;
        if Self::striker_death_check(left, right, world) {
            return Ok(());
        }
        let units = vec![(left, -1.0), (right, 1.0)];
        GameTimer::get().start_batch();
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
            GameTimer::get().to_batch_start();
        }
        GameTimer::get().insert_to_end().end_batch();
        ActionPlugin::spin(world)?;
        Ok(())
    }

    fn strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("Strike {left:?} {right:?}");
        let units = vec![(left, right), (right, left)];
        for (caster, target) in units {
            let context = Context::from_caster(caster, world)
                .set_target(target, world)
                .set_owner(caster, world)
                .take();
            let effect = Effect::Damage(None);
            ActionPlugin::action_push_back(effect, context, world);
        }
        GameTimer::get().advance_insert(0.3);
        ActionPlugin::spin(world)?;
        Ok(())
    }

    fn after_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("After strike {left:?} {right:?}");
        let units = vec![left, right];
        GameTimer::get().start_batch();
        for caster in units {
            Options::get_animations(world)
                .get(AnimationType::AfterStrike)
                .clone()
                .apply(Context::from_owner(caster, world), world)
                .unwrap();
            GameTimer::get().to_batch_start();
        }
        GameTimer::get().insert_to_end().end_batch();
        Event::AfterStrike(left, right).send(world);
        ActionPlugin::spin(world)?;
        Event::AfterStrike(right, left).send(world);
        ActionPlugin::spin(world)?;
        Ok(())
    }

    fn fatigue(world: &mut World) -> Result<()> {
        let (turn, _) = ActionPlugin::get_turn(GameTimer::get().insert_head(), world);
        let fatigue = if let Some(settings) = GlobalSettings::filter_by_always_zero(0) {
            settings.fatigue_start
        } else {
            20
        } as i32;
        let fatigue = turn as i32 - fatigue;
        if fatigue > 0 {
            info!("Fatigue {fatigue}");
            let effect = Effect::Damage(Some(Expression::Int(fatigue)));
            for (unit, _) in
                UnitPlugin::collect_factions([Faction::Left, Faction::Right].into(), world)
            {
                ActionPlugin::action_push_back(
                    effect.clone(),
                    Context::from_owner(unit, world)
                        .set_target(unit, world)
                        .take(),
                    world,
                );
            }
        }
        Ok(())
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        if !GameTimer::get().ended() {
            let bd = world.resource::<BattleData>();
            Self::draw_player_names(bd, ctx);
            Self::draw_round_turn_num(bd, ctx, world);
            Self::draw_current_event(ctx, world);
            return;
        }
        Self::draw_final_panel(ctx, world);
    }

    fn draw_player_names(bd: &BattleData, ctx: &egui::Context) {
        if let Some(left) = &bd.left_player_data {
            Area::new("left player".into())
                .anchor(Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        left.name
                            .add_color(white())
                            .set_style(ColoredStringStyle::Heading)
                            .label(ui);
                        format!("#{}", left.id).to_colored().label(ui);
                    });
                });
        }
        if let Some(right) = &bd.right_player_data {
            Area::new("right player".into())
                .anchor(Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        right
                            .name
                            .add_color(white())
                            .set_style(ColoredStringStyle::Heading)
                            .label(ui);
                        format!("#{}", right.id).to_colored().label(ui);
                    });
                });
        }
    }

    fn draw_round_turn_num(bd: &BattleData, ctx: &egui::Context, world: &World) {
        let (turn, ts) = ActionPlugin::get_turn(GameTimer::get().play_head(), world);
        let x = smoothstep(0.2, 0.0, ts);
        let spacing = x * 4.0;
        TopBottomPanel::top("turn num").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if bd.run_id.is_some() {
                    if let Some(run) = ArenaRun::current() {
                        format!("Round {}", run.round)
                            .add_color(orange())
                            .set_style(ColoredStringStyle::Heading)
                            .set_extra_spacing(spacing)
                            .label(ui);
                    }
                }
                format!("Turn {turn}")
                    .add_color(orange())
                    .set_style(ColoredStringStyle::Heading2)
                    .set_extra_spacing(spacing)
                    .label(ui)
            })
        });
    }

    fn draw_current_event(ctx: &egui::Context, world: &World) {
        if let Some((event, ts)) = ActionPlugin::get_event(world) {
            let x = smoothstep(0.2, 0.0, ts);
            let spacing = x * 4.0;
            TopBottomPanel::top("event text").show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    let text = event
                        .to_string()
                        .add_color(yellow())
                        .set_style(ColoredStringStyle::Heading2)
                        .set_extra_spacing(spacing)
                        .rich_text(ui);
                    ui.heading(text);
                });
            });
        }
    }

    fn draw_final_panel(ctx: &egui::Context, world: &mut World) {
        let bd = world.resource::<BattleData>();
        if let Some(win) = bd.result.is_win() {
            let text = match win {
                true => "Victory".to_owned(),
                false => "Defeat".to_owned(),
            };
            let mut subtext = "".to_colored();
            let mut run_active = false;
            if let Some(id) = bd.run_id {
                let data = if let Some(run) = ArenaRun::filter_by_id(id) {
                    run_active = true;
                    Some((run.wins(), run.loses()))
                } else if let Some(run) = ArenaArchive::filter_by_id(id) {
                    subtext.push("Run Over\n\n".to_owned(), yellow());
                    Some((run.wins as usize, run.loses as usize))
                } else {
                    None
                };
                if let Some((wins, loses)) = data {
                    subtext
                        .push("Wins: ".to_owned(), light_gray())
                        .push(format!("{}", wins), white())
                        .push("\nLoses: ".to_owned(), light_gray())
                        .push(format!("{}/3", loses), red());
                }
            }
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
                        text.add_color(color)
                            .set_style_ref(ColoredStringStyle::Heading)
                            .label(ui);
                        subtext.label(ui);
                        ui.columns(2, |ui| {
                            ui[0].vertical_centered_justified(|ui| {
                                if ui.button("REPLAY").clicked() {
                                    let t = -AudioPlugin::to_next_beat(world);
                                    GameTimer::get().play_head_to(t);
                                }
                            });
                            ui[1].vertical_centered_justified(|ui| {
                                if ui.button_primary("OK").clicked() {
                                    match run_active {
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

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
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

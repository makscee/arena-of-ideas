use egui::ImageButton;

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), Self::on_exit)
            .add_systems(OnEnter(GameState::CustomBattle), Self::on_enter_custom)
            .add_systems(OnEnter(GameState::ShopBattle), Self::on_enter_shop)
            .init_resource::<BattleData>();
    }
}

impl BattlePlugin {
    fn on_enter(world: &mut World) {
        info!("Start battle");
        gt().reset();
        let result = Self::run(world).unwrap();
        let mut bd = world.resource_mut::<BattleData>();
        bd.result = result;
        if bd.id > 0 {
            submit_battle_result(match result {
                BattleResult::Tbd => TBattleResult::Tbd,
                BattleResult::Left(_) => TBattleResult::Left,
                BattleResult::Right(_) => TBattleResult::Right,
                BattleResult::Even => TBattleResult::Even,
            });
            once_on_submit_battle_result(|_, _, status, _| match status {
                StdbStatus::Committed => {}
                StdbStatus::Failed(e) => {
                    error!("Battle result submit error: {e}")
                }
                _ => panic!(),
            });
        }
        info!("Battle finished with result: {result:?}");
    }
    fn on_exit(world: &mut World) {
        gt().reset();
        world.game_clear();
    }
    fn on_enter_custom(world: &mut World) {
        world.insert_resource(GameAssets::get(world).custom_battle.clone());
        GameState::Battle.set_next(world);
    }
    fn on_enter_shop(world: &mut World) {
        let run = TArenaRun::current();
        let bid = *run.battles.last().unwrap();
        let battle = TBattle::filter_by_id(bid).unwrap();
        let left = PackedTeam::from_id(battle.team_left);
        let right = PackedTeam::from_id(battle.team_right);
        world.insert_resource(BattleData {
            left,
            right,
            id: bid,
            ..default()
        });
        GameState::Battle.set_next(world);
    }
    pub fn load_teams(left: PackedTeam, right: PackedTeam, world: &mut World) {
        world.insert_resource(BattleData {
            left,
            right,
            ..default()
        });
    }
    pub fn run(world: &mut World) -> Result<BattleResult> {
        let bd = world.resource::<BattleData>();
        let left = bd.left.clone();
        let right = bd.right.clone();
        left.unpack(Faction::Left, world);
        right.unpack(Faction::Right, world);
        UnitPlugin::fill_gaps_and_translate(world);
        ActionPlugin::spin(world)?;
        Event::BattleStart.send(world);
        ActionPlugin::spin(world)?;
        ActionPlugin::spin(world)?;
        loop {
            if let Some((left, right)) = Self::get_strikers(world) {
                Self::run_strike(left, right, world)?;
                continue;
            } else {
                debug!("no strikers");
            }
            if ActionPlugin::spin(world)? || ActionPlugin::clear_dead(world) {
                continue;
            }
            break;
        }
        Self::get_result(world)
    }
    pub fn clear(world: &mut World) {
        for unit in UnitPlugin::collect_all(world) {
            world.entity_mut(unit).despawn_recursive();
        }
    }
    fn get_strikers(world: &mut World) -> Option<(Entity, Entity)> {
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
    fn run_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        ActionPlugin::spin(world)?;
        ActionPlugin::register_next_turn(world);
        Event::TurnStart.send(world);
        ActionPlugin::spin(world)?;
        Self::before_strike(left, right, world)?;
        if Self::striker_death_check(left, right, world) {
            return Ok(());
        }
        Self::strike(left, right, world)?;
        Self::after_strike(left, right, world)?;
        Event::TurnEnd.send(world);
        ActionPlugin::spin(world)?;
        ActionPlugin::spin(world)?;
        Ok(())
    }
    fn before_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("before strike {left:?} {right:?}");
        Event::BeforeStrike(left, right).send(world);
        ActionPlugin::spin(world)?;
        Event::BeforeStrike(right, left).send(world);
        ActionPlugin::spin(world)?;
        if Self::striker_death_check(left, right, world) {
            return Ok(());
        }
        let units = vec![(left, -1.0), (right, 1.0)];
        let mut shift: f32 = 0.0;
        for (caster, dir) in units {
            shift = shift.max(
                GameAssets::get(world)
                    .animations
                    .before_strike
                    .clone()
                    .apply(
                        Context::new(caster)
                            .set_var(VarName::Direction, VarValue::Float(dir))
                            .take(),
                        world,
                    )
                    .unwrap(),
            );
        }
        gt().advance_insert(shift);
        ActionPlugin::spin(world)?;
        Ok(())
    }
    fn strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("strike {left:?} {right:?}");
        let units = vec![(left, right), (right, left)];
        for (caster, target) in units {
            let context = Context::new(caster)
                .set_target(target)
                .set_caster(caster)
                .take();
            let effect = Effect::Damage;
            ActionPlugin::action_push_back(effect, context, world);
        }
        ActionPlugin::spin(world)?;
        Ok(())
    }
    fn after_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("after strike {left:?} {right:?}");
        UnitPlugin::translate_to_slots(world);
        Event::AfterStrike(left, right).send(world);
        ActionPlugin::spin(world)?;
        Event::AfterStrike(right, left).send(world);
        ActionPlugin::spin(world)?;
        Ok(())
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
    pub fn widgets(ctx: &egui::Context, world: &mut World) {
        TopMenu::new(vec!["Playback"]).show(ctx);
        Tile::bottom("Playback")
            .transparent()
            .non_resizable()
            .content(|ui, world| {
                ui.vertical_centered(|ui| {
                    let mut gt = gt();
                    if ImageButton::new(if gt.paused() {
                        Icon::Pause.image()
                    } else {
                        Icon::Play.image()
                    })
                    .ui(ui)
                    .clicked()
                    {
                        let paused = gt.paused();
                        gt.pause(!paused);
                    }
                });

                Middle3::default().ui_mut(
                    ui,
                    world,
                    |ui, _| {
                        format!("{:.2}", gt().play_head())
                            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading)
                            .label(ui);
                    },
                    |ui, world| {
                        const FF_LEFT_KEY: &str = "ff_back_btn";
                        let pressed = get_context_bool(world, FF_LEFT_KEY);
                        if pressed {
                            gt().advance_play(-delta_time(world) * 2.0);
                        }
                        let resp = ImageButton::new(Icon::FFBack.image())
                            .tint(if pressed { YELLOW } else { VISIBLE_BRIGHT })
                            .ui(ui);
                        set_context_bool(
                            world,
                            FF_LEFT_KEY,
                            resp.contains_pointer() && left_mouse_pressed(world),
                        );
                    },
                    |ui, world| {
                        const FF_RIGHT_KEY: &str = "ff_forward_btn";
                        let pressed = get_context_bool(world, FF_RIGHT_KEY);
                        if pressed {
                            gt().advance_play(delta_time(world));
                        }
                        let resp = ImageButton::new(Icon::FFForward.image())
                            .tint(if pressed { YELLOW } else { VISIBLE_BRIGHT })
                            .ui(ui);
                        set_context_bool(
                            world,
                            FF_RIGHT_KEY,
                            resp.contains_pointer() && left_mouse_pressed(world),
                        );
                    },
                );
                Middle3::default().width(400.0).ui_mut(
                    ui,
                    world,
                    |ui, _| {
                        Slider::new("Playback Speed")
                            .log()
                            .name(false)
                            .range(-20.0..=20.0)
                            .ui(&mut gt().playback_speed, ui);
                    },
                    |ui, _| {
                        if ImageButton::new(Icon::SkipBack.image()).ui(ui).clicked() {
                            gt().play_head_to(0.0);
                        }
                    },
                    |ui, _| {
                        if ImageButton::new(Icon::SkipForward.image()).ui(ui).clicked() {
                            gt().skip_to_end();
                        }
                    },
                );
            })
            .show(ctx, world);
    }
    pub fn ui(ui: &mut Ui, world: &mut World) {
        if !gt().ended() {
            return;
        }
        center_window("end_panel", ui, |ui| {
            ui.vertical_centered(|ui| {
                let bd = world.resource::<BattleData>();
                if bd.result.is_win().unwrap_or_default() {
                    "Victory".cstr_cs(GREEN, CstrStyle::Bold)
                } else {
                    "Defeat".cstr_cs(RED, CstrStyle::Bold)
                }
                .label(ui);
                if Button::click("Finish".into()).ui(ui).clicked() {
                    GameState::Shop.proceed_to_target(world);
                }
            });
        });
    }
}

#[derive(Asset, TypePath, Resource, Default, Clone, Debug, Deserialize)]
pub struct BattleData {
    #[serde(default)]
    id: u64,
    left: PackedTeam,
    right: PackedTeam,
    #[serde(default)]
    result: BattleResult,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Display)]
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

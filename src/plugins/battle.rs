use egui::ImageButton;

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), Self::on_exit)
            .add_systems(OnEnter(GameState::CustomBattle), Self::on_enter_custom)
            .add_systems(OnEnter(GameState::ShopBattle), Self::on_enter_shop)
            .add_systems(
                Update,
                Self::hover_check.run_if(in_state(GameState::Battle)),
            )
            .add_systems(Update, Self::input.run_if(in_state(GameState::Battle)))
            .init_resource::<BattleResource>();
    }
}

impl BattlePlugin {
    fn on_enter(world: &mut World) {
        info!("Start battle");
        gt().reset();
        let result = Self::run(world).unwrap();
        let mut bd = world.resource_mut::<BattleResource>();
        bd.result = result;
        if bd.id > 0 && bd.from_run {
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
    }
    fn on_exit(world: &mut World) {
        gt().reset();
        world.game_clear();
        ActionPlugin::reset(world);
    }
    fn on_enter_custom(world: &mut World) {
        world.insert_resource(game_assets().custom_battle.clone());
        GameState::Battle.set_next(world);
    }
    fn on_enter_shop(world: &mut World) {
        let run = TArenaRun::current();
        let bid = *run.battles.last().unwrap();
        let battle = TBattle::find_by_id(bid).unwrap();
        let mut bd = BattleResource::from(battle);
        bd.from_run = true;
        bd.next_state = GameState::Shop;
        world.insert_resource(bd);
        GameState::Battle.set_next(world);
    }
    pub fn set_next_state(state: GameState, world: &mut World) {
        rm(world).next_state = state;
    }
    pub fn load_teams(left: PackedTeam, right: PackedTeam, world: &mut World) {
        world.insert_resource(BattleResource {
            left,
            right,
            ..default()
        });
    }
    pub fn run(world: &mut World) -> Result<BattleResult> {
        ActionPlugin::reset(world);
        let bd = world.resource::<BattleResource>();
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
        let result = Self::get_result(world);
        info!("Battle finished with result: {result:?}");
        if let Ok(result) = result.as_ref() {
            if result.is_win().unwrap_or_default() {
                ActionPlugin::register_sound_effect(SoundEffect::Victory, world);
            } else {
                ActionPlugin::register_sound_effect(SoundEffect::Defeat, world);
            }
        }
        result
    }
    pub fn clear(world: &mut World) {
        for unit in UnitPlugin::collect_all(world) {
            world.entity_mut(unit).despawn_recursive();
        }
    }
    pub fn get_strikers(world: &mut World) -> Option<(Entity, Entity)> {
        if let Some(left) = UnitPlugin::find_unit(Faction::Left, 0, world) {
            if let Some(right) = UnitPlugin::find_unit(Faction::Right, 0, world) {
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
        let (turn, _) = ActionPlugin::get_turn(gt().insert_head(), world);
        Self::fatigue(turn, world)
    }
    fn before_strike(left: Entity, right: Entity, world: &mut World) -> Result<()> {
        debug!("before strike {left} {right}");
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
                game_assets()
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
        debug!("strike {left} {right}");
        let units = vec![(left, right), (right, left)];
        game_assets()
            .animations
            .strike
            .clone()
            .apply(Context::default(), world)?;
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
        debug!("after strike {left} {right}");
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
            let faction = Context::new(unit).get_faction(world)?;
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
    fn fatigue(turn: usize, world: &mut World) -> Result<()> {
        let fatigue = game_assets().global_settings.battle.fatigue_start as usize;
        if turn <= fatigue {
            return Ok(());
        }
        let fatigue = turn - fatigue;
        info!("Fatigue {fatigue}");
        let dmg = fatigue * fatigue;
        for unit in UnitPlugin::collect_alive(world) {
            let context = Context::new(unit)
                .set_target(unit)
                .set_var(VarName::Value, dmg.into())
                .take();
            ActionPlugin::action_push_back(Effect::Damage, context, world);
            ActionPlugin::spin(world)?;
        }
        Ok(())
    }
    fn input(input: Res<ButtonInput<KeyCode>>, time: Res<Time>) {
        if input.just_pressed(KeyCode::Space) {
            let paused = gt().paused();
            gt().pause(!paused);
        }
        if input.pressed(KeyCode::ArrowLeft) {
            gt().advance_play(-time.delta_seconds() * 2.0);
        }
        if input.pressed(KeyCode::ArrowRight) {
            gt().advance_play(time.delta_seconds() * 2.0);
        }
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Bottom, |ui, world| {
            ui.vertical_centered(|ui| {
                let mut gt = gt();
                if ImageButton::new(if gt.paused() {
                    Icon::Play.image()
                } else {
                    Icon::Pause.image()
                })
                .rounding(13.0)
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
                        .cstr_cs(visible_bright(), CstrStyle::Heading)
                        .label(ui);
                },
                |ui, world| {
                    const FF_LEFT_KEY: &str = "ff_back_btn";
                    let pressed = get_context_bool(world, FF_LEFT_KEY);
                    if pressed {
                        gt().advance_play(-delta_time(world) * 2.0);
                    }
                    let resp = ImageButton::new(Icon::FFBack.image())
                        .rounding(13.0)
                        .tint(if pressed { YELLOW } else { visible_bright() })
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
                        .rounding(13.0)
                        .tint(if pressed { YELLOW } else { visible_bright() })
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
                    Slider::new("Playback Speed").log().name(false).ui(
                        &mut gt().playback_speed,
                        -20.0..=20.0,
                        ui,
                    );
                },
                |ui, _| {
                    if ImageButton::new(Icon::SkipBack.image())
                        .rounding(13.0)
                        .ui(ui)
                        .clicked()
                    {
                        gt().play_head_to(0.0);
                    }
                },
                |ui, _| {
                    if ImageButton::new(Icon::SkipForward.image())
                        .rounding(13.0)
                        .ui(ui)
                        .clicked()
                    {
                        gt().skip_to_end();
                    }
                },
            );
        })
        .non_focusable()
        .transparent()
        .pinned()
        .push(world);

        let bd = rm(world);
        if bd.id == 0 {
            return;
        }
        if let Some(battle) = TBattle::find_by_id(bd.id) {
            let show_team = |team: TTeam, ui: &mut Ui| {
                if team.id == 0 {
                    return;
                }
                text_dots_text("owner".cstr(), team.owner.get_user().cstr(), ui);
                if !team.name.is_empty() {
                    text_dots_text(
                        "team name".cstr(),
                        team.name.cstr_cs(visible_light(), CstrStyle::Bold),
                        ui,
                    );
                }
                text_dots_text(
                    "team id".cstr(),
                    team.id.to_string().cstr_c(visible_light()),
                    ui,
                );
            };
            Tile::new(Side::Left, move |ui, _| {
                show_team(battle.team_left.get_team(), ui);
            })
            .transparent()
            .pinned()
            .non_focusable()
            .no_frame()
            .push(world);
            Tile::new(Side::Right, move |ui, _| {
                show_team(battle.team_right.get_team(), ui);
            })
            .transparent()
            .pinned()
            .non_focusable()
            .no_frame()
            .push(world);
        }
    }
    pub fn ui(ui: &mut Ui, world: &mut World) {
        if !gt().ended() {
            return;
        }
        popup("end_panel", ui.ctx(), |ui| {
            ui.vertical_centered_justified(|ui| {
                let bd = world.resource::<BattleResource>();
                if bd.result.is_win().unwrap_or_default() {
                    "Victory".cstr_cs(GREEN, CstrStyle::Heading2)
                } else {
                    "Defeat".cstr_cs(RED, CstrStyle::Heading2)
                }
                .label(ui);
            });
            space(ui);
            ui.columns(2, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if Button::click("Replay").gray(ui).ui(ui).clicked() {
                        gt().play_head_to(0.0);
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    if Button::click("Finish").ui(ui).clicked() {
                        rm(world).next_state.proceed_to_target(world);
                    }
                });
            })
        });
    }
    pub fn hover_check(world: &mut World) {
        if gt().ended() {
            return;
        }
        let Some(cursor_pos) = cursor_world_pos(world) else {
            return;
        };
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        for unit in UnitPlugin::collect_all(world) {
            let context = Context::new_play(unit);
            if !context.get_bool(VarName::Visible, world).unwrap_or(true) {
                continue;
            }
            let pos = context
                .get_vec2(VarName::Position, world)
                .unwrap_or_default();
            if (pos - cursor_pos).length() < 1.0 {
                cursor_window(ctx, |ui| match UnitCard::new(&context, world) {
                    Ok(c) => c.ui(ui),
                    Err(e) => error!("{e}"),
                });
                return;
            }
        }
    }
}

#[derive(Asset, TypePath, Resource, Default, Clone, Debug, Deserialize)]
pub struct BattleResource {
    #[serde(default)]
    id: u64,
    left: PackedTeam,
    right: PackedTeam,
    #[serde(default)]
    result: BattleResult,
    #[serde(default)]
    from_run: bool,
    #[serde(default = "default_state")]
    next_state: GameState,
}
fn rm(world: &mut World) -> Mut<BattleResource> {
    world.resource_mut::<BattleResource>()
}

fn default_state() -> GameState {
    GameState::Title
}

impl From<TBattle> for BattleResource {
    fn from(value: TBattle) -> Self {
        Self {
            id: value.id,
            left: PackedTeam::from_id(value.team_left),
            right: PackedTeam::from_id(value.team_right),
            result: BattleResult::Tbd,
            from_run: false,
            next_state: GameState::Shop,
        }
    }
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
            BattleResult::Left(..) => Some(true),
            BattleResult::Right(..) | BattleResult::Even => Some(false),
        }
    }
}

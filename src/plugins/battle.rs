use egui::ImageButton;

use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), Self::on_exit)
            .add_systems(OnEnter(GameState::CustomBattle), Self::on_enter_custom)
            .init_resource::<BattleData>();
    }
}

impl BattlePlugin {
    fn on_enter(world: &mut World) {
        info!("Start battle");
        GameTimer::get().reset();
        let result = Self::run(world).unwrap();
        let mut bd = world.resource_mut::<BattleData>();
        bd.result = result;
        info!("Battle finished with result: {result:?}");
    }
    fn on_exit(world: &mut World) {
        GameTimer::get().reset();
        world.game_clear();
    }
    fn on_enter_custom(world: &mut World) {
        world.insert_resource(GameAssets::get(world).custom_battle.clone());
    }
    pub fn load_teams(left: PackedTeam, right: PackedTeam, world: &mut World) {
        world.insert_resource(BattleData {
            left,
            right,
            result: default(),
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
        GameTimer::get().advance_insert(shift);
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
            .content(|ui, world| {
                ui.vertical_centered(|ui| {
                    let mut gt = GameTimer::get();
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

                Middle3::default().ui(
                    ui,
                    world,
                    |ui, _| {
                        format!("{:.2}", GameTimer::get().play_head())
                            .cstr_cs(WHITE, CstrStyle::Heading)
                            .label(ui);
                    },
                    |ui, world| {
                        const FF_LEFT_KEY: &str = "ff_back_btn";
                        let pressed = get_context_bool(world, FF_LEFT_KEY);
                        if pressed {
                            GameTimer::get().advance_play(-delta_time(world) * 2.0);
                        }
                        let resp = ImageButton::new(Icon::FFBack.image())
                            .tint(if pressed { YELLOW } else { WHITE })
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
                            GameTimer::get().advance_play(delta_time(world));
                        }
                        let resp = ImageButton::new(Icon::FFForward.image())
                            .tint(if pressed { YELLOW } else { WHITE })
                            .ui(ui);
                        set_context_bool(
                            world,
                            FF_RIGHT_KEY,
                            resp.contains_pointer() && left_mouse_pressed(world),
                        );
                    },
                );
                Middle3::default().width(400.0).ui(
                    ui,
                    world,
                    |ui, _| {
                        Slider::new("Playback Speed")
                            .log()
                            .name(false)
                            .range(-20.0..=20.0)
                            .ui(&mut GameTimer::get().playback_speed, ui);
                    },
                    |ui, _| {
                        if ImageButton::new(Icon::SkipBack.image()).ui(ui).clicked() {
                            GameTimer::get().play_head_to(0.0);
                        }
                    },
                    |ui, _| {
                        if ImageButton::new(Icon::SkipForward.image()).ui(ui).clicked() {
                            GameTimer::get().skip_to_end();
                        }
                    },
                );
            })
            .show(ctx, world);
    }
}

#[derive(Asset, TypePath, Resource, Default, Clone, Debug, Deserialize)]
pub struct BattleData {
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

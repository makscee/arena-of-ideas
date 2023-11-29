use bevy_egui::egui::{Align2, Window};

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
}

impl BattlePlugin {
    pub fn on_enter(world: &mut World) {
        GameTimer::get_mut(world).reset();
        let result = Self::run_battle(100, world).unwrap();
        world.resource_mut::<BattleData>().result = result;
        match result {
            BattleResult::Left(_) | BattleResult::Even => {
                let mut sd = Save::get(world).unwrap();
                sd.current_level += 1;
                sd.save(world).unwrap();
            }
            BattleResult::Right(_) => {
                let mut save = Save::get(world).unwrap();
                save.team = default();
                save.current_level = 0;
                save.save(world).unwrap();
                let mut pd = PersistentData::load(world);
                if pd.last_state == Some(GameState::Shop) {
                    pd.last_state = None;
                }
                pd.save(world).unwrap();
            }
            BattleResult::Tbd => panic!("Battle result fail"),
        }
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
        });
    }

    pub fn on_leave(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
    }

    pub fn run_battle(bpm: usize, world: &mut World) -> Result<BattleResult> {
        ActionPlugin::set_timeframe(60.0 / bpm as f32, world);
        const SHIFT_LEFT: f32 = -1.5;
        GameTimer::get_mut(world)
            .advance_insert(SHIFT_LEFT)
            .advance_play(SHIFT_LEFT);
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
        Window::new("")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::LEFT_CENTER, [10.0, 100.0])
            .show(&egui_context(world), |ui| {
                if ui
                    .button(
                        RichText::new("Skip")
                            .size(20.0)
                            .text_style(egui::TextStyle::Button),
                    )
                    .clicked()
                {
                    GameTimer::get_mut(world).skip_to_end();
                }
            });
        if !GameTimer::get(world).ended() {
            return;
        }
        let bd = world.resource::<BattleData>();
        let victory = match bd.result {
            BattleResult::Left(_) | BattleResult::Even => true,
            BattleResult::Right(_) => false,
            BattleResult::Tbd => panic!("No battle result found"),
        };
        let (text, color) = match victory {
            true => ("Victory".to_owned(), hex_color!("#00E5FF")),
            false => (
                format!("Defeat. Reached level {}", bd.level.unwrap_or_default() + 1),
                hex_color!("#FF1744"),
            ),
        };
        Window::new(
            RichText::new(text)
                .color(color)
                .size(30.0)
                .text_style(egui::TextStyle::Heading),
        )
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, -150.0])
        .show(&egui_context(world), |ui| {
            ui.set_min_width(300.0);
            ui.vertical_centered_justified(|ui| {
                if ui
                    .button(
                        RichText::new("Replay")
                            .size(20.0)
                            .color(hex_color!("#78909C"))
                            .text_style(egui::TextStyle::Button),
                    )
                    .clicked()
                {
                    GameTimer::get_mut(world).play_head_to(0.0);
                }
                if ui
                    .button(
                        RichText::new("Ok")
                            .size(20.0)
                            .text_style(egui::TextStyle::Button),
                    )
                    .clicked()
                {
                    if victory {
                        GameState::change(GameState::Shop, world);
                    } else {
                        GameState::change(GameState::MainMenu, world);
                    }
                }
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

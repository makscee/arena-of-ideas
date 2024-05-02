use std::thread::sleep;

use crate::module_bindings::{run_change_g, ArenaRun, TeamUnit};

use self::module_bindings::{GlobalSettings, ShopOffer, User};

use super::*;

use bevy::input::common_conditions::input_just_pressed;
use bevy_egui::egui::Order;

pub struct ShopPlugin;

#[derive(Resource, Clone)]
pub struct ShopData {
    update_callback: UpdateCallbackId<ArenaRun>,
    phase: ShopPhase,
}

#[derive(Clone, PartialEq)]
pub enum ShopPhase {
    None {
        stack: HashMap<Entity, Vec<Entity>>,
        fuse: HashMap<Entity, Vec<Entity>>,
    },
    FuseStart {
        source: Entity,
        targets: Vec<Entity>,
    },
    FuseEnd {
        source: Entity,
        target: Entity,
        candidates: Vec<PackedUnit>,
    },
    Stack {
        source: Entity,
        targets: Vec<Entity>,
    },
}

impl ShopPhase {
    fn initial(world: &mut World) -> Self {
        let sources = UnitPlugin::collect_all(world);
        let targets = UnitPlugin::collect_faction(Faction::Team, world);
        let (stack, fuse) = UnitPlugin::collect_merge_targets(sources, targets, world);
        Self::None { stack, fuse }
    }
}

impl ShopData {
    pub fn is_initial_phase(&self) -> bool {
        matches!(self.phase, ShopPhase::None { .. })
    }
    pub fn show_other_ui(&self) -> bool {
        match self.phase {
            ShopPhase::None { .. } | ShopPhase::FuseStart { .. } | ShopPhase::Stack { .. } => true,
            ShopPhase::FuseEnd { .. } => false,
        }
    }
}

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::on_enter)
            .add_systems(OnExit(GameState::Shop), Self::on_exit)
            .add_systems(
                OnTransition {
                    from: GameState::Shop,
                    to: GameState::Battle,
                },
                Self::transition_to_battle,
            )
            .add_systems(PostUpdate, Self::input.run_if(in_state(GameState::Shop)))
            .add_systems(
                Update,
                ((
                    Self::ui.after(PanelsPlugin::ui),
                    Self::win.run_if(input_just_pressed(KeyCode::KeyV)),
                    Self::fuse_front.run_if(input_just_pressed(KeyCode::KeyF)),
                )
                    .run_if(in_state(GameState::Shop)),),
            );
    }
}

impl ShopPlugin {
    fn win(world: &mut World) {
        ServerOperation::SubmitResult(true).send(world).unwrap();
        OperationsPlugin::add(|w| {
            Self::on_exit(w);
            Self::on_enter(w);
        });
    }

    fn on_enter(world: &mut World) {
        GameTimer::get().reset();
        TeamPlugin::spawn(Faction::Shop, world);
        TeamPlugin::spawn(Faction::Team, world);
        UnitPlugin::translate_to_slots(world);
        // So there's enough time for subscription if we run straight into Shop state
        if Self::load_state(world).is_err() {
            sleep(Duration::from_secs_f32(0.1));
        } else {
            return;
        }
        if Self::load_state(world).is_err() {
            GameState::MainMenu.change(world);
        }
    }

    fn fuse_front(world: &mut World) {
        let entity_a = UnitPlugin::find_unit(Faction::Team, 1, world).unwrap();
        let entity_b = UnitPlugin::find_unit(Faction::Team, 2, world).unwrap();
        Self::start_fuse(entity_a, entity_b, world);
    }

    pub fn start_fuse(source: Entity, target: Entity, world: &mut World) {
        world.resource_mut::<ShopData>().phase = ShopPhase::FuseEnd {
            source,
            target,
            candidates: Self::get_fuse_candidates(source, target, world),
        }
    }

    fn get_fuse_candidates(source: Entity, target: Entity, world: &mut World) -> Vec<PackedUnit> {
        let a = PackedUnit::pack(target, world);
        let b = PackedUnit::pack(source, world);
        PackedUnit::fuse(a, b, world)
    }

    fn on_exit(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
        ArenaRun::remove_on_update(world.resource::<ShopData>().update_callback.clone());
    }

    pub fn load_next_battle(world: &mut World) {
        let run = ArenaRun::current().unwrap();
        BattlePlugin::load_from_run(run, world);
    }

    fn transition_to_battle(world: &mut World) {
        Self::load_next_battle(world);
        let mut pd = PersistentData::load(world);
        pd.last_battle = world.resource::<BattleData>().clone();
        pd.save(world).unwrap();
    }

    fn input(world: &mut World) {
        if just_pressed(KeyCode::KeyG, world) {
            run_change_g(10);
        }
    }

    fn load_state(world: &mut World) -> Result<()> {
        let update_callback = ArenaRun::on_update(|_, new, event| {
            if !new.user_id.eq(&LoginPlugin::get_user_data().unwrap().id) {
                return;
            }
            debug!("ArenaRun callback: {event:?}");
            let new = new.clone();
            OperationsPlugin::add(move |world| {
                Self::sync_units(&new.state.team, Faction::Team, world);
                Self::sync_units_state(&new.state.team, Faction::Team, world);
                Self::sync_units(&new.get_case_units(), Faction::Shop, world);
                Self::sync_offers(&new.state.case, world);
                let phase = ShopPhase::initial(world);
                if let Some(mut data) = world.get_resource_mut::<ShopData>() {
                    data.phase = phase;
                }
            })
        });
        let run = ArenaRun::current().context("No active run")?;
        Self::sync_units(&run.state.team, Faction::Team, world);
        Self::sync_units(&run.get_case_units(), Faction::Shop, world);
        Self::sync_offers(&run.state.case, world);
        debug!("Shop insert data");
        let phase = ShopPhase::initial(world);
        world.insert_resource(ShopData {
            update_callback,
            phase,
        });
        Ok(())
    }

    fn sync_units(units: &Vec<TeamUnit>, faction: Faction, world: &mut World) {
        debug!("Start sync {} {faction}", units.len());
        let world_units = UnitPlugin::collect_faction_ids(faction, world);
        let team = TeamPlugin::find_entity(faction, world).unwrap();
        for unit in units {
            if world_units.contains_key(&unit.id) {
                continue;
            }
            let id = unit.id;
            let unit: PackedUnit = unit.unit.clone().into();
            let unit = unit.unpack(team, None, world);
            VarState::get_mut(unit, world).set_int(VarName::Id, id as i32);
        }
        let world_units = UnitPlugin::collect_faction(faction, world);
        if world_units.len() > units.len() {
            for unit in world_units {
                let id = VarState::get(unit, world).get_int(VarName::Id).unwrap() as u64;
                if !units.iter().any(|u| u.id.eq(&id)) {
                    world.entity_mut(unit).despawn_recursive();
                }
            }
        }
        UnitPlugin::fill_slot_gaps(faction, world);
        UnitPlugin::translate_to_slots(world);
    }

    fn sync_units_state(units: &Vec<TeamUnit>, faction: Faction, world: &mut World) {
        let world_units = UnitPlugin::collect_faction_ids(faction, world);
        for TeamUnit { id, unit } in units {
            let entity = world_units.get(id).unwrap();
            let mut state = VarState::get_mut(*entity, world);
            state.set_int(VarName::Hp, unit.hp);
            state.set_int(VarName::Pwr, unit.pwr);
            state.set_int(VarName::Stacks, unit.stacks);
            state.set_int(VarName::Level, unit.level);
            state.set_string(VarName::Houses, unit.houses.clone());
        }
    }

    fn sync_offers(offers: &Vec<ShopOffer>, world: &mut World) {
        let gs = GlobalSettings::filter_by_always_zero(0).unwrap();
        let world_units = UnitPlugin::collect_faction_ids(Faction::Shop, world);
        for ShopOffer {
            available: _,
            discount,
            freeze,
            unit,
        } in offers
        {
            let price = if *discount {
                gs.price_unit_sell
            } else {
                gs.price_unit_buy
            };
            if let Some(entity) = world_units.get(&unit.id) {
                let mut state = VarState::get_mut(*entity, world);
                state.set_int(VarName::Price, price as i32);
                state.set_bool(VarName::Freeze, *freeze);
            }
        }
    }

    pub fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let mut data = world.remove_resource::<ShopData>().unwrap();
        if matches!(data.phase, ShopPhase::FuseEnd { .. }) {
            Self::show_fusion_options(&mut data, world);
            world.insert_resource(data);
            return;
        }

        let pos = UnitPlugin::get_slot_position(Faction::Shop, 0) - vec2(1.0, 0.0);
        let pos = world_to_screen(pos.extend(0.0), world);
        let pos = pos2(pos.x, pos.y);

        // Self::draw_buy_panels(world);
        let _ = Self::show_hero_ui(&mut data, world);
        Self::show_info_table(world);

        Area::new("reroll".into()).fixed_pos(pos).show(ctx, |ui| {
            ui.set_width(120.0);
            ui.set_enabled(!ServerPlugin::pending(world).is_some());
            frame(ui, |ui| {
                "Reroll".add_color(white()).label(ui);
                let run = ArenaRun::current().unwrap();
                let text = if run.state.free_rerolls > 0 {
                    format!("-0g ({})", run.state.free_rerolls)
                } else {
                    format!(
                        "-{}g",
                        GlobalSettings::filter_by_always_zero(0)
                            .unwrap()
                            .price_reroll
                    )
                };
                let text = text.add_color(yellow()).rich_text(ui).size(16.0);
                if ui.button(text).clicked() {
                    Self::buy_reroll(world);
                }
            });
        });

        let g = ArenaRun::current().unwrap().state.g;
        Area::new("g".into())
            .fixed_pos(pos + egui::vec2(0.0, -60.0))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("{g} g"))
                        .size(40.0)
                        .strong()
                        .color(hex_color!("#FFC107")),
                );
            });
        Area::new("battle button".into())
            .anchor(Align2::RIGHT_CENTER, [-40.0, -50.0])
            .show(ctx, |ui| {
                ui.set_enabled(!ServerPlugin::pending(world).is_some());
                if ui.button("START BATTLE").clicked() {
                    Self::go_to_battle(world);
                }
            });
        match &data.phase {
            ShopPhase::FuseStart { source, targets } => {
                if let Ok(target) = targets.iter().exactly_one() {
                    data.phase = ShopPhase::FuseEnd {
                        source: *source,
                        target: *target,
                        candidates: Self::get_fuse_candidates(*source, *target, world),
                    };
                }
            }
            ShopPhase::Stack { source, targets } => {
                if let Ok(target) = targets.iter().exactly_one() {
                    UnitPlugin::stack_units(*target, *source, world);
                    data.phase = ShopPhase::initial(world);
                }
            }
            _ => {}
        }
        world.insert_resource(data);
    }

    fn show_fusion_options(data: &mut ShopData, world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        match &mut data.phase {
            ShopPhase::FuseEnd {
                source,
                target,
                candidates,
            } => {
                let len = candidates.len();
                window("CHOOSE FUSION")
                    .order(Order::Foreground)
                    .set_width(len as f32 * 240.0)
                    .anchor(Align2::CENTER_TOP, [0.0, 40.0])
                    .show(ctx, |ui| {
                        ui.columns(len, |ui| {
                            for (i, fusion) in candidates.iter().enumerate() {
                                let state = fusion.generate_state(world);
                                let statuses = fusion.statuses.clone();
                                frame(&mut ui[i], |ui| {
                                    state.show_state_card_ui(i, statuses, true, ui, world);
                                });
                            }
                        });
                        ui.columns(len, |ui| {
                            for i in 0..len {
                                ui[i].vertical_centered(|ui| {
                                    frame(ui, |ui| {
                                        if ui.button("ACCEPT").clicked() {
                                            let fused = candidates.remove(i).into();
                                            ServerOperation::Fuse {
                                                a: *source,
                                                b: *target,
                                                fused,
                                            }
                                            .send(world)
                                            .unwrap();
                                            candidates.clear();
                                        }
                                    });
                                });
                            }
                        });
                        frame(ui, |ui| {
                            ui.set_width(300.0);
                            if ui.button_red("CANCEL").clicked() {
                                candidates.clear();
                            }
                        });
                    });
                if candidates.is_empty() {
                    data.phase = ShopPhase::initial(world);
                }
            }
            _ => {}
        }
    }

    fn show_info_table(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let run = ArenaRun::current().expect("Current run not found");
        window("INFO")
            .anchor(Align2::LEFT_TOP, [10.0, 10.0])
            .show(ctx, |ui| {
                frame(ui, |ui| {
                    text_dots_text(
                        &"name".to_colored(),
                        &format!("{}", User::filter_by_id(run.user_id).unwrap().name)
                            .add_color(white()),
                        ui,
                    );
                    text_dots_text(
                        &"round".to_colored(),
                        &run.round.to_string().add_color(white()),
                        ui,
                    );
                    text_dots_text(
                        &"wins".to_colored(),
                        &run.wins().to_string().add_color(white()),
                        ui,
                    );
                    text_dots_text(
                        &"loses".to_colored(),
                        &run.loses().to_string().add_color(white()),
                        ui,
                    );
                });
            });
    }

    fn show_hero_ui(data: &mut ShopData, world: &mut World) -> Result<()> {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return Ok(());
        };
        let cursor_pos = CameraPlugin::cursor_world_pos(world).context("Failed to get cursor")?;
        let dragged = world.resource::<DraggedUnit>().0;
        let gs = GlobalSettings::filter_by_always_zero(0).unwrap();
        if let Some((dragged, _)) = dragged {
            let mut new_action = DragAction::None;
            for slot in 1..=gs.team_slots as usize {
                let pos = UnitPlugin::get_slot_position(Faction::Team, slot);
                if (pos - cursor_pos).length() < 1.0 {
                    new_action = DragAction::Insert(slot);
                }
            }
            world.resource_mut::<DraggedUnit>().0 = Some((dragged, new_action));
        } else {
            let units = UnitPlugin::collect_factions([Faction::Team, Faction::Shop].into(), world);
            let phase = data.phase.clone();
            for (entity, faction) in units {
                let is_shop = faction == Faction::Shop;
                let state = VarState::get(entity, world);
                let price = state.get_int(VarName::Price).unwrap_or_default() as i64;
                let freeze = state.get_bool(VarName::Freeze).unwrap_or_default();
                let discount = if is_shop && price < gs.price_unit_buy {
                    Some("discount".add_color(yellow()))
                } else {
                    None
                };
                let offset = &mut (vec2(0.0, -2.7));
                match &phase {
                    ShopPhase::None { stack, fuse } => {
                        if let Some(stack) = stack.get(&entity) {
                            if !stack.is_empty() {
                                let text = if is_shop {
                                    format!("Stack -{} g", gs.price_unit_buy_stack.min(price))
                                } else {
                                    "Stack".to_owned()
                                };
                                let resp = Self::draw_unit_button(
                                    entity,
                                    offset,
                                    orange(),
                                    &text,
                                    &discount,
                                    false,
                                    |_| {
                                        data.phase = ShopPhase::Stack {
                                            source: entity,
                                            targets: stack.clone(),
                                        };
                                    },
                                    ctx,
                                    world,
                                );
                                let rect = resp.rect;
                                if resp
                                    .on_hover_text_at_pointer(
                                        "Stack heroes together,\nget +1 HP and +1 PWR per stack\nand increase Level",
                                    )
                                    .hovered()
                                {
                                    Self::draw_arrows(rect.center_top(), stack, ctx, world);
                                }
                            }
                        }
                        if let Some(fuse) = fuse.get(&entity) {
                            if !fuse.is_empty() {
                                let resp = Self::draw_unit_button(
                                    entity,
                                    offset,
                                    red(),
                                    "Fuse",
                                    &None,
                                    false,
                                    |_| {
                                        data.phase = ShopPhase::FuseStart {
                                            source: entity,
                                            targets: fuse.clone(),
                                        };
                                    },
                                    ctx,
                                    world,
                                );
                                let rect = resp.rect;
                                if resp.on_hover_text("Fuse heroes together, combining their abilities\nHeroes of Level greater than 1 can be fused").hovered() {
                                    Self::draw_arrows(rect.center_top(), fuse, ctx, world);
                                }
                            }
                        }

                        if is_shop {
                            Self::draw_unit_button(
                                entity,
                                offset,
                                yellow(),
                                &format!("-{} g", gs.price_unit_buy.min(price)),
                                &discount.clone().or_else(|| Some("buy".to_colored())),
                                false,
                                |world| {
                                    ServerOperation::Buy(entity).send(world).unwrap();
                                },
                                ctx,
                                world,
                            );
                            Self::draw_unit_button(
                                entity,
                                &mut vec2(0.0, 1.0),
                                if freeze { yellow() } else { white() },
                                if freeze { "unfreeze" } else { "freeze" },
                                &None,
                                true,
                                |world| {
                                    ServerOperation::Freeze(entity).send(world).unwrap();
                                },
                                ctx,
                                world,
                            );
                        } else {
                            let state = VarState::try_get(entity, world)?;
                            if state.get_int(VarName::Slot).context("Failed to get slot")?
                                == UnitPlugin::get_closest_slot(cursor_pos, Faction::Team).0 as i32
                            {
                                Self::draw_unit_button(
                                    entity,
                                    offset,
                                    yellow(),
                                    &format!("+{} g", gs.price_unit_sell),
                                    &Some("sell".to_colored()),
                                    false,
                                    |world| {
                                        ServerOperation::Sell(entity).send(world).unwrap();
                                    },
                                    ctx,
                                    world,
                                );
                            }
                        }
                    }
                    ShopPhase::FuseStart { source, targets }
                    | ShopPhase::Stack { source, targets } => {
                        let is_stack = matches!(&data.phase, ShopPhase::Stack { .. });
                        if entity.eq(&source) {
                            Self::draw_unit_button(
                                entity,
                                offset,
                                red(),
                                &format!("Cancel {}", if is_stack { "Stack" } else { "Fuse" }),
                                &None,
                                false,
                                |world| {
                                    data.phase = ShopPhase::initial(world);
                                },
                                ctx,
                                world,
                            );
                        } else if targets.contains(&entity) {
                            Self::draw_unit_button(
                                entity,
                                offset,
                                yellow(),
                                "Choose",
                                &None,
                                false,
                                |world| {
                                    if is_stack {
                                        UnitPlugin::stack_units(entity, *source, world)
                                    } else {
                                        data.phase = ShopPhase::FuseEnd {
                                            source: *source,
                                            target: entity,
                                            candidates: Self::get_fuse_candidates(
                                                *source, entity, world,
                                            ),
                                        };
                                    }
                                },
                                ctx,
                                world,
                            );
                        }
                    }
                    ShopPhase::FuseEnd { .. } => {}
                }
            }
        }

        Ok(())
    }
    fn draw_unit_button(
        entity: Entity,
        offset: &mut Vec2,
        color: Color32,
        text: &str,
        label: &Option<ColoredString>,
        small: bool,
        action: impl FnOnce(&mut World),
        ctx: &egui::Context,
        world: &mut World,
    ) -> Response {
        const OFFSET_DELTA: Vec2 = vec2(0.0, -1.0);
        if label.is_some() {
            *offset += OFFSET_DELTA * 0.5;
        }
        window(text)
            .id(entity)
            .set_width(120.0)
            .title_bar(false)
            .stroke(false)
            .entity_anchor(entity, Align2::CENTER_BOTTOM, *offset, world)
            .show(ctx, |ui| {
                ui.set_enabled(!ServerPlugin::pending(world).is_some());
                *offset += OFFSET_DELTA;
                frame(ui, |ui| {
                    ui.set_width(100.0);
                    if let Some(label) = label {
                        label.label(ui);
                    }
                    let text = text
                        .add_color(color)
                        .set_style(if small {
                            ColoredStringStyle::Small
                        } else {
                            ColoredStringStyle::Normal
                        })
                        .rich_text(ui);
                    let resp = ui.button(text);
                    if resp.clicked() {
                        action(world);
                    }
                    resp
                })
            })
            .response
    }
    fn draw_arrows(from: Pos2, targets: &Vec<Entity>, ctx: &egui::Context, world: &World) {
        let screen_rect = ctx.screen_rect();
        egui::Window::new("curves")
            .fixed_rect(screen_rect)
            .frame(Frame::none())
            .title_bar(false)
            .show(ctx, |ui| {
                const OFFSET: egui::Vec2 = egui::vec2(0.0, 40.0);
                let p1 = from;
                let p2 = p1 - OFFSET;
                for target in targets {
                    let p4 = entity_screen_pos(*target, vec2(0.0, 0.0), world).to_pos2();
                    let p3 = p4 + OFFSET;
                    draw_curve(p1, p2, p3, p4, 3.0, red(), true, ui);
                }
            });
    }

    fn go_to_battle(world: &mut World) {
        GameState::change(GameState::Battle, world);
    }

    pub fn buy_reroll(world: &mut World) {
        ServerOperation::Reroll.send(world).unwrap();
    }
}

pub trait ArenaRunExt {
    fn get_case_units(&self) -> Vec<TeamUnit>;
    fn current() -> Option<ArenaRun>;
    fn wins(&self) -> usize;
    fn loses(&self) -> usize;
}

impl ArenaRunExt for ArenaRun {
    fn get_case_units(&self) -> Vec<TeamUnit> {
        self.state
            .case
            .iter()
            .filter_map(|o| {
                if o.available {
                    Some(o.unit.clone())
                } else {
                    None
                }
            })
            .collect_vec()
    }
    fn current() -> Option<Self> {
        LoginPlugin::get_user_data().and_then(|user| Self::filter_by_user_id(user.id))
    }

    fn wins(&self) -> usize {
        self.battles
            .iter()
            .filter(|b| b.result.is_some_and(|r| r))
            .count()
    }

    fn loses(&self) -> usize {
        self.battles
            .iter()
            .filter(|b| b.result.is_some_and(|r| !r))
            .count()
    }
}

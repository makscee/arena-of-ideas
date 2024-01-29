use std::thread::sleep;

use crate::module_bindings::{
    run_buy, run_change_g, run_fuse, run_reroll, run_sell, run_submit_result, ArenaPool, ArenaRun,
    TeamUnit,
};

use super::*;

use bevy::input::common_conditions::input_just_pressed;
use bevy_egui::egui::Order;

pub struct ShopPlugin;

#[derive(Resource, Clone)]
pub struct ShopData {
    update_callback: UpdateCallbackId<ArenaRun>,
    pub fusion_candidates: Option<((Entity, Entity), Vec<PackedUnit>)>,
}

const REROLL_PRICE: i32 = 1;

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
                    Self::win.run_if(input_just_pressed(KeyCode::V)),
                    Self::fuse_front.run_if(input_just_pressed(KeyCode::F)),
                )
                    .run_if(in_state(GameState::Shop)),),
            );
    }
}

impl ShopPlugin {
    fn win() {
        run_submit_result(true);
        OperationsPlugin::add(|w| {
            Self::on_exit(w);
            Self::on_enter(w);
        });
    }

    fn on_enter(world: &mut World) {
        GameTimer::get().reset();
        PackedTeam::spawn(Faction::Shop, world);
        PackedTeam::spawn(Faction::Team, world);
        let update_callback = ArenaRun::on_update(|_, new, event| {
            debug!("ArenaRun callback: {event:?}");
            let new = new.clone();
            OperationsPlugin::add(move |world| {
                Self::sync_units(&new.state.team, Faction::Team, world);
                Self::sync_units_state(&new.state.team, Faction::Team, world);
                Self::sync_units(&new.get_case_units(), Faction::Shop, world);
            })
        });
        UnitPlugin::translate_to_slots(world);
        ActionPlugin::set_timeframe(0.05, world);
        debug!("Shop insert data");
        world.insert_resource(ShopData {
            update_callback,
            fusion_candidates: default(),
        });
        // So there's enough time for subscription if we run staight into Shop state
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

    pub fn start_fuse(entity_a: Entity, entity_b: Entity, world: &mut World) {
        let a = PackedUnit::pack(entity_a, world);
        let b = PackedUnit::pack(entity_b, world);
        let fusions = PackedUnit::fuse(a, b, world);
        world.resource_mut::<ShopData>().fusion_candidates = Some(((entity_a, entity_b), fusions));
    }

    fn on_exit(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
        ArenaRun::remove_on_update(world.resource::<ShopData>().update_callback.clone());
    }

    fn transition_to_battle(world: &mut World) {
        let run = ArenaRun::filter_by_active(true).next().unwrap();
        let left =
            PackedTeam::from_table_units(run.state.team.into_iter().map(|u| u.unit).collect());
        let right = if let Some(right) = run.enemies.last() {
            let right = ArenaPool::filter_by_id(*right).unwrap().team;
            PackedTeam::from_table_units(right)
        } else {
            default()
        };
        BattlePlugin::load_teams(left, right, Some(run.id), world);
    }

    fn input(world: &mut World) {
        if just_pressed(KeyCode::G, world) {
            run_change_g(10);
        }
    }

    fn load_state(world: &mut World) -> Result<()> {
        let run = ArenaRun::filter_by_active(true)
            .next()
            .context("No active run")?;
        Self::sync_units(&run.state.team, Faction::Team, world);
        Self::sync_units(&run.get_case_units(), Faction::Shop, world);
        Ok(())
    }

    fn sync_units(units: &Vec<TeamUnit>, faction: Faction, world: &mut World) {
        debug!("Start sync {} {faction}", units.len());
        let world_units = UnitPlugin::collect_faction_ids(faction, world);
        let team = PackedTeam::find_entity(faction, world).unwrap();
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
            state.set_int(VarName::Atk, unit.atk);
            state.set_int(VarName::Stacks, unit.stacks);
            state.set_int(VarName::Level, unit.level);
            // state.set_string(VarName::AbilityDescription, unit.description.clone());
            state.set_string(VarName::Houses, unit.houses.clone());
        }
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut data = world.remove_resource::<ShopData>().unwrap();
        if data.fusion_candidates.is_some() {
            Self::show_fusion_options(&mut data, world);
            world.insert_resource(data);
            return;
        }

        let pos = UnitPlugin::get_slot_position(Faction::Shop, 0) - vec2(1.0, 0.0);
        let pos = world_to_screen(pos.extend(0.0), world);
        let pos = pos2(pos.x, pos.y);

        Self::draw_buy_panels(world);
        let _ = Self::show_hero_ui(world);
        Self::show_info_table(world);

        Area::new("reroll").fixed_pos(pos).show(ctx, |ui| {
            ui.set_width(120.0);
            frame(ui, |ui| {
                "Reroll".add_color(white()).label(ui);
                let text = format!("-{}g", REROLL_PRICE)
                    .add_color(yellow())
                    .rich_text(ui)
                    .size(20.0);
                if ui.button(text).clicked() {
                    Self::buy_reroll();
                }
            });
        });

        let g = ArenaRun::filter_by_active(true).next().unwrap().state.g;
        Area::new("g")
            .fixed_pos(pos + egui::vec2(0.0, -60.0))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("{g} g"))
                        .size(40.0)
                        .strong()
                        .color(hex_color!("#FFC107")),
                );
            });
        Area::new("battle button")
            .anchor(Align2::RIGHT_CENTER, [-40.0, 0.0])
            .show(ctx, |ui| {
                if ui.button("START BATTLE").clicked() {
                    Self::go_to_battle(world);
                }
            });
        world.insert_resource(data);
    }

    fn show_fusion_options(data: &mut ShopData, world: &mut World) {
        let ctx = &egui_context(world);
        if let Some(((entity_a, entity_b), candidates)) = &mut data.fusion_candidates {
            let len = candidates.len();
            window("CHOOSE FUSION")
                .order(Order::Foreground)
                .set_width(len as f32 * 240.0)
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
                                        let fused = candidates.remove(i);
                                        let a = UnitPlugin::get_id(*entity_a, world).unwrap();
                                        let b = UnitPlugin::get_id(*entity_b, world).unwrap();
                                        run_fuse(a, b, fused.into());
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
                data.fusion_candidates = None;
            }
        }
    }

    fn show_info_table(world: &mut World) {
        let run = ArenaRun::current().expect("Current run not found");
        window("INFO")
            .anchor(Align2::LEFT_TOP, [10.0, 10.0])
            .show(&egui_context(world), |ui| {
                frame(ui, |ui| {
                    text_dots_text(
                        &"wins".to_colored(),
                        &run.state.wins.to_string().add_color(white()),
                        ui,
                    );
                    text_dots_text(
                        &"loses".to_colored(),
                        &run.state.loses.to_string().add_color(white()),
                        ui,
                    );
                });
            })
    }

    fn show_hero_ui(world: &mut World) -> Result<()> {
        let ctx = &egui_context(world);
        let cursor_pos = CameraPlugin::cursor_world_pos(world).context("Failed to get cursor")?;
        let dragged = world.resource::<DraggedUnit>().0;
        if let Some((dragged, action)) = dragged {
            let mut new_action = DragAction::None;
            let dragged_state = VarState::get(dragged, world);
            let dragged_houses = dragged_state.get_houses_vec()?;
            let dragged_level = dragged_state.get_int(VarName::Level)?;
            for entity in UnitPlugin::collect_faction(Faction::Team, world) {
                if entity == dragged {
                    continue;
                }
                let state = VarState::get(entity, world);
                let houses = state.get_houses_vec()?;
                let level = state.get_int(VarName::Level)?;
                let same_house = dragged_houses.iter().any(|h| houses.contains(h));
                if same_house {
                    let stacks = state.get_int(VarName::Stacks)?;
                    let level = state.get_int(VarName::Level)?;
                    let color = if matches!(action, DragAction::Stack(e) if e == entity) {
                        yellow()
                    } else {
                        white()
                    };
                    window("STACK")
                        .id(entity)
                        .set_width(150.0)
                        .title_bar(false)
                        .stroke(false)
                        .entity_anchor(entity, Align2::CENTER_BOTTOM, vec2(0.0, 2.2), world)
                        .show(ctx, |ui| {
                            if frame(ui, |ui| {
                                "+STACK"
                                    .add_color(color)
                                    .set_style(ColoredStringStyle::Heading)
                                    .label(ui);
                                format!("Level {level}").add_color(color).label(ui);

                                format!("{stacks}/{}", level + 1)
                                    .add_color(light_gray())
                                    .label(ui);
                            })
                            .response
                            .hovered()
                            {
                                new_action = DragAction::Stack(entity);
                            };
                        });
                } else if level > houses.len() as i32
                    && houses.len() < 3
                    && dragged_level >= level
                    && dragged_houses.len() == 1
                    && !houses.contains(&dragged_houses[0])
                {
                    let color = if matches!(action, DragAction::Fuse(e) if e == entity) {
                        yellow()
                    } else {
                        white()
                    };
                    window("FUSE")
                        .id(entity)
                        .set_width(150.0)
                        .title_bar(false)
                        .stroke(false)
                        .entity_anchor(entity, Align2::CENTER_BOTTOM, vec2(0.0, 2.2), world)
                        .show(ctx, |ui| {
                            if frame(ui, |ui| {
                                "FUSE"
                                    .add_color(color)
                                    .set_style(ColoredStringStyle::Heading)
                                    .label(ui);
                            })
                            .response
                            .hovered()
                            {
                                new_action = DragAction::Fuse(entity);
                            };
                        });
                }
            }
            for slot in 1..TEAM_SLOTS {
                let pos = UnitPlugin::get_slot_position(Faction::Team, slot);
                if (pos - cursor_pos).length() < 1.0 {
                    new_action = DragAction::Insert(slot);
                }
            }
            world.resource_mut::<DraggedUnit>().0 = Some((dragged, new_action));
        } else {
            for entity in UnitPlugin::collect_faction(Faction::Team, world) {
                let state = VarState::get(entity, world);
                if state.get_int(VarName::Slot).context("Failed to get slot")?
                    == UnitPlugin::get_closest_slot(cursor_pos, Faction::Team).0 as i32
                {
                    window("SELL")
                        .id(entity)
                        .set_width(120.0)
                        .title_bar(false)
                        .stroke(false)
                        .entity_anchor(entity, Align2::CENTER_BOTTOM, vec2(0.0, 2.0), world)
                        .show(ctx, |ui| {
                            frame(ui, |ui| {
                                ui.set_width(100.0);
                                ui.label("sell");
                                let text = "+1 g".add_color(yellow()).rich_text(ui).size(20.0);
                                if ui.button(text).clicked() {
                                    run_sell(
                                        VarState::get(entity, world).get_int(VarName::Id).unwrap()
                                            as u64,
                                    );
                                    world.entity_mut(entity).despawn_recursive();
                                    UnitPlugin::fill_slot_gaps(Faction::Team, world);
                                    UnitPlugin::translate_to_slots(world);
                                }
                            });
                        });
                }
            }
        }

        Ok(())
    }
    fn draw_buy_panels(world: &mut World) {
        let ctx = &egui_context(world);
        let run = ArenaRun::filter_by_active(true).next().unwrap();
        let units = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Shop, world)
                .into_iter()
                .map(|unit| {
                    (
                        VarState::get(unit, world).get_int(VarName::Id).unwrap() as u64,
                        unit,
                    )
                }),
        );
        for offer in run.state.case {
            let id = offer.unit.id;

            if let Some(entity) = units.get(&id) {
                window("BUY")
                    .id(entity)
                    .set_width(120.0)
                    .title_bar(false)
                    .stroke(false)
                    .entity_anchor(*entity, Align2::CENTER_TOP, vec2(0.0, -1.2), world)
                    .show(ctx, |ui| {
                        // ui.set_enabled(
                        //     offer.available
                        //         && save.climb.shop.can_afford(offer.price)
                        //         && save.climb.team.units.len() < TEAM_SLOTS,
                        // );
                        frame(ui, |ui| {
                            ui.set_width(100.0);
                            let text = format!("-{} g", offer.price)
                                .add_color(yellow())
                                .rich_text(ui)
                                .size(20.0);
                            if ui.button(text).clicked() {
                                run_buy(id);
                            }
                        });
                    });
            }
        }
    }

    fn go_to_battle(world: &mut World) {
        GameState::change(GameState::Battle, world);
    }

    pub fn buy_reroll() {
        run_reroll(false);
    }
}

pub trait ArenaRunExt {
    fn get_case_units(self) -> Vec<TeamUnit>;
    fn current() -> Option<ArenaRun>;
}

impl ArenaRunExt for ArenaRun {
    fn get_case_units(self) -> Vec<TeamUnit> {
        self.state
            .case
            .into_iter()
            .filter_map(|o| if o.available { Some(o.unit) } else { None })
            .collect_vec()
    }
    fn current() -> Option<Self> {
        ArenaRun::filter_by_active(true).next()
    }
}

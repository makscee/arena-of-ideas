use std::str::FromStr;

use bevy::input::common_conditions::input_just_pressed;
use spacetimedb_sdk::table::{TableWithPrimaryKey, UpdateCallbackId};

use super::*;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::enter)
            .add_systems(OnExit(GameState::Shop), Self::exit)
            .init_resource::<ShopData>();
        if cfg!(debug_assertions) {
            app.add_systems(
                Update,
                Self::give_g.run_if(input_just_pressed(KeyCode::KeyG)),
            );
        }
    }
}

#[derive(Resource, Clone)]
pub struct ShopData {
    pub case_height: f32,
    callback: Option<UpdateCallbackId<Run>>,
}

impl Default for ShopData {
    fn default() -> Self {
        Self {
            case_height: 0.0,
            callback: None,
        }
    }
}

impl ShopPlugin {
    fn give_g() {
        shop_change_g(10);
    }
    fn enter(mut sd: ResMut<ShopData>) {
        if let Some(run) = Run::get_current() {
            if !run.active {
                GameState::GameOver.set_next_op();
                return;
            }
            OperationsPlugin::add(|world| Self::sync_run(*run, world));
        } else {
            run_start();
            once_on_run_start(|_, _, status| match status {
                spacetimedb_sdk::reducer::Status::Committed => OperationsPlugin::add(|world| {
                    Self::sync_run(Run::current(), world);
                }),
                spacetimedb_sdk::reducer::Status::Failed(e) => {
                    Notification::new(format!("Run start error: {e}"))
                        .error()
                        .push_op()
                }
                _ => panic!(),
            });
        }
        let cb = Run::on_update(|_, run, _| {
            let run = run.clone();
            OperationsPlugin::add(|world| Self::sync_run(run, world))
        });
        sd.callback = Some(cb);
    }
    fn exit(world: &mut World) {
        world.game_clear();
        if let Some(cb) = world.resource_mut::<ShopData>().callback.take() {
            Run::remove_on_update(cb);
        }
    }
    fn sync_run(run: Run, world: &mut World) {
        debug!("Sync run");
        let mut shop_units: HashMap<u64, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Shop, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Shop, world);
        for (
            i,
            ShopSlot {
                unit,
                available,
                id,
                ..
            },
        ) in run.shop.into_iter().enumerate()
        {
            let slot = i as i32 + 1;
            if available {
                if shop_units.contains_key(&id) {
                    shop_units.remove(&id);
                    continue;
                }
                GameAssets::get(world)
                    .heroes
                    .get(&unit)
                    .cloned()
                    .unwrap()
                    .unpack(team, Some(slot), Some(id), world);
            }
        }
        for entity in shop_units.into_values() {
            UnitPlugin::despawn(entity, world);
        }
        let mut team_units: HashMap<u64, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Team, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Team, world);
        for (i, unit) in TTeam::filter_by_id(run.team)
            .unwrap()
            .units
            .into_iter()
            .enumerate()
        {
            let id = unit.id;
            let slot = i as i32 + 1;
            if let Some(entity) = team_units.get(&id) {
                let mut state = VarState::get_mut(*entity, world);
                state.set_int(VarName::Slot, slot.into());
                state.set_int(VarName::Stacks, unit.stacks as i32);
                team_units.remove(&id);
                continue;
            }
            let unit: PackedUnit = unit.into();
            unit.unpack(team, Some(slot), Some(id), world);
        }
        for entity in team_units.into_values() {
            UnitPlugin::despawn(entity, world);
        }
    }
    pub fn widgets(ctx: &egui::Context, world: &mut World) {
        TopMenu::new(vec!["Container Config"]).show(ctx);
        Tile::left("Container Config")
            .content(|ui, world| {
                let mut data = world.resource_mut::<ShopData>();
                Slider::new("offset")
                    .range(-100.0..=400.0)
                    .ui(&mut data.case_height, ui);
            })
            .show(ctx, world);
    }
    pub fn overlay_widgets(ctx: &egui::Context, world: &mut World) {
        Tile::left("Stats")
            .open()
            .transparent()
            .non_resizable()
            .content(|ui, _| {
                text_dots_text(&"name".cstr(), &user_name().cstr_c(WHITE), ui);
                if let Some(run) = Run::get_current() {
                    text_dots_text(
                        &"G".cstr(),
                        &run.g.to_string().cstr_cs(YELLOW, CstrStyle::Bold),
                        ui,
                    );
                    text_dots_text(&"round".cstr(), &run.round.to_string().cstr_c(WHITE), ui);
                }
            })
            .show(ctx, world);
        Tile::right("To battle")
            .open()
            .transparent()
            .non_resizable()
            .content(|ui, _| {
                if Button::click("Start Battle".into()).ui(ui).clicked() {
                    shop_finish();
                    once_on_shop_finish(|_, _, status| match status {
                        spacetimedb_sdk::reducer::Status::Committed => {
                            GameState::ShopBattle.proceed_to_target_op()
                        }
                        spacetimedb_sdk::reducer::Status::Failed(e) => {
                            error!("Failed to finish shop: {e}")
                        }
                        _ => panic!(),
                    });
                }
            })
            .show(ctx, world);
    }
    pub fn show_containers(wd: &mut WidgetData, ui: &mut Ui, world: &mut World) {
        if let Some(run) = Run::get_current() {
            if let Some((_, pos)) = ui.ctx().get_dragged() {
                if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                    ui.painter().arrow(
                        pos,
                        pointer.to_vec2() - pos.to_vec2(),
                        Stroke {
                            width: 3.0,
                            color: YELLOW,
                        },
                    )
                }
            }
            if let Some((from, to)) = ui.ctx().drag_finished() {
                let from = from.split("/").collect_vec();
                let from_faction = Faction::from_str(from[from.len() - 2]).unwrap();
                let from_slot = usize::from_str(from[from.len() - 1]).unwrap();
                let to = to.split("/").collect_vec();
                let to_faction = Faction::from_str(to[to.len() - 2]).unwrap();
                let to_slot = usize::from_str(to[to.len() - 1]).unwrap();
                if matches!(from_faction, Faction::Team) && from_faction.eq(&to_faction) {
                    shop_reorder(from_slot as u8, to_slot as u8);
                }
            }

            let sd = world.resource::<ShopData>().clone();
            let shop = run.shop;
            let team = TTeam::filter_by_id(run.team).unwrap();
            let g = run.g;
            UnitContainer::new(Faction::Shop)
                .direction(Side::Top)
                .offset([0.0, -sd.case_height])
                .slots(shop.len())
                .top_content(move |ui, _| {
                    if Button::click(format!("-1 G"))
                        .title("Reroll".into())
                        .enabled(g >= 1)
                        .ui(ui)
                        .clicked()
                    {
                        shop_reroll();
                    }
                })
                .slot_content(move |slot, _, ui, _| {
                    let ind = slot - 1;
                    let ss = &shop[ind];
                    if ss.available {
                        if Button::click(format!("-{} G", ss.price))
                            .title("buy".into())
                            .enabled(g >= ss.price)
                            .ui(ui)
                            .clicked()
                        {
                            shop_buy(slot as u8);
                        }
                        if !ss.stack_targets.is_empty() {
                            let price = ss.price - 1;
                            if Button::click(format!("-{} G", price))
                                .title("stack".into())
                                .enabled(g >= price)
                                .ui(ui)
                                .clicked()
                            {
                                stack_shop(slot as u8, ss.stack_targets[0]);
                                once_on_stack_shop(|_, _, status, _, _| match status {
                                    StdbStatus::Committed => {}
                                    StdbStatus::Failed(e) => {
                                        Notification::new(format!("Stack failed: {e}"))
                                            .error()
                                            .push_op()
                                    }
                                    _ => panic!(),
                                });
                            }
                        }
                    }
                })
                .hover_content(Self::container_on_hover)
                .ui(wd, ui, world);
            let slots = GameAssets::get(world).global_settings.team_slots as usize;
            UnitContainer::new(Faction::Team)
                .direction(Side::Bottom)
                .offset([0.0, sd.case_height])
                .slots(slots.max(team.units.len()))
                .max_slots(slots)
                .slot_content(move |slot, e, ui, _| {
                    let ind = slot - 1;
                    if e.is_some() {
                        if Button::click("+1 G".into())
                            .title("Sell".into())
                            .ui(ui)
                            .clicked()
                        {
                            shop_sell(ind as u8);
                        }
                    }
                })
                .hover_content(Self::container_on_hover)
                .ui(wd, ui, world);
        }
    }
    fn container_on_hover(_: usize, entity: Option<Entity>, ui: &mut Ui, world: &mut World) {
        let Some(entity) = entity else {
            return;
        };
        let state = VarState::get(entity, world);
        let t = gt().play_head();
        unit_card(t, state, ui);
    }
    pub fn game_over_ui(ui: &mut Ui) {
        let Some(run) = Run::get_current() else {
            return;
        };
        center_window("game_over", ui, |ui| {
            ui.set_width(300.0);
            ui.vertical_centered_justified(|ui| {
                if run.lives > 0 {
                    "Victory".cstr_cs(GREEN, CstrStyle::Heading)
                } else {
                    "Defeat".cstr_cs(RED, CstrStyle::Heading)
                }
                .label(ui);
                "Run Over".cstr_cs(YELLOW, CstrStyle::Bold).label(ui);
                format!("Final round: ")
                    .cstr()
                    .push(run.round.to_string().cstr_cs(YELLOW, CstrStyle::Bold))
                    .label(ui);
                if Button::click("Finish".into()).ui(ui).clicked() {
                    run_finish();
                    once_on_run_finish(|_, _, status| match status {
                        StdbStatus::Committed => GameState::Title.proceed_to_target_op(),
                        StdbStatus::Failed(e) => error!("Failed to finish run: {e}"),
                        _ => panic!(),
                    });
                }
            });
        });
    }
}

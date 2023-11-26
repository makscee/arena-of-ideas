use super::*;

use rand::seq::IteratorRandom;

pub struct ShopPlugin;

#[derive(Resource, Clone)]
pub struct ShopData {
    pub next_team: PackedTeam,
    pub next_level_num: usize,
    pub phase: ShopPhase,
}

#[derive(Clone, PartialEq)]
pub enum ShopPhase {
    Buy,
    Sacrifice { selected: HashSet<usize> },
}

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                from: GameState::MainMenu,
                to: GameState::Shop,
            },
            Self::on_enter,
        )
        .add_systems(OnExit(GameState::Shop), Self::on_leave)
        .add_systems(
            OnTransition {
                from: GameState::Battle,
                to: GameState::Shop,
            },
            Self::level_finished.before(Self::on_enter),
        )
        .add_systems(PostUpdate, Self::input.run_if(in_state(GameState::Shop)))
        .add_systems(Update, (Self::ui.run_if(in_state(GameState::Shop)),));
    }
}

impl ShopPlugin {
    pub const UNIT_PRICE: i32 = 3;
    pub const REROLL_PRICE: i32 = 1;

    fn on_enter(world: &mut World) {
        let save = Save::get(world).unwrap();
        if Ladder::is_on_last_level(world) {
            let teams =
                RatingPlugin::generate_weakest_opponent(&Save::get(world).unwrap().team, 3, world);
            Save::get(world)
                .unwrap()
                .add_ladder_levels(teams)
                .save(world)
                .unwrap();
        }
        let team_len = save.team.units.len();
        save.team.unpack(Faction::Team, world);
        Self::change_g(4, world).unwrap();
        UnitPlugin::translate_to_slots(world);
        Self::fill_showcase(world);
        PersistentData::load(world)
            .set_last_state(GameState::Shop)
            .save(world)
            .unwrap();
        let phase = match team_len < SACRIFICE_SLOT {
            true => ShopPhase::Buy,
            false => ShopPhase::Sacrifice {
                selected: default(),
            },
        };
        let (next_team, next_level_num) = Ladder::current_level(world);
        world.insert_resource(ShopData {
            next_team,
            next_level_num: next_level_num + 1,
            phase,
        });
    }

    fn level_finished(world: &mut World) {
        if Ladder::is_on_last_level(world) {
            let teams =
                RatingPlugin::generate_weakest_opponent(&Save::get(world).unwrap().team, 3, world);
            Save::get(world)
                .unwrap()
                .add_ladder_levels(teams)
                .save(world)
                .unwrap();
        }
        Self::on_enter(world);
    }

    fn on_leave(world: &mut World) {
        Self::pack_active_team(world).unwrap();
        UnitPlugin::despawn_all_teams(world);
        Self::clear_showcase(world);

        let left = Self::active_team(world).unwrap();
        let (right, ind) = Ladder::current_level(world);
        BattlePlugin::load_teams(left, right, Some(ind), world);
    }

    fn input(world: &mut World) {
        if just_pressed(KeyCode::G, world) {
            Self::change_g(10, world).unwrap();
        }
    }

    fn fill_showcase(world: &mut World) {
        let mut units = Vec::default();
        for _ in 0..3 {
            let unit = Pools::get(world)
                .heroes
                .values()
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();
            units.push(unit);
        }
        let team = PackedTeam::spawn(Faction::Shop, world);
        let units_len = units.len();
        for unit in units {
            let description = unit.description.to_owned();
            let unit = unit.unpack(team, None, world);
            world.entity_mut(unit).insert(ShopOffer {
                name: "Hero".to_owned(),
                description,
                price: Self::UNIT_PRICE,
                product: OfferProduct::Unit,
            });
        }
        UnitPlugin::fill_slot_gaps(Faction::Shop, world);
        UnitPlugin::translate_to_slots(world);

        for i in 1..3 {
            let pos = UnitPlugin::get_slot_position(Faction::Shop, units_len + i as usize);
            let status = Pools::get_status("Strength", world).unwrap().clone();
            let name = status.name.to_owned();
            let description = status.description.to_owned();
            let charges = status.state.get_int(VarName::Charges).unwrap_or(1);
            let entity = status.unpack(None, world);
            VarState::get_mut(entity, world).init(VarName::Position, VarValue::Vec2(pos));
            world.entity_mut(entity).insert(ShopOffer {
                product: OfferProduct::Status {
                    name: name.to_owned(),
                    charges,
                },
                name,
                description,
                price: 2,
            });
        }
    }

    fn clear_showcase(world: &mut World) {
        for entity in Self::all_offers(world) {
            world.entity_mut(entity).despawn_recursive();
        }
    }

    pub fn pack_active_team(world: &mut World) -> Result<()> {
        let team = PackedTeam::pack(Faction::Team, world);
        Save::get(world)?
            .set_team(team)
            .save(world)
            .map_err(|e| anyhow!("{}", e.to_string()))
    }

    pub fn active_team(world: &mut World) -> Result<PackedTeam> {
        Ok(Save::get(world)?.team)
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut data = world.resource::<ShopData>().clone();
        let mut sacrifice_accepted = false;

        let pos = UnitPlugin::get_slot_position(Faction::Shop, 0);
        let pos = world_to_screen(pos.extend(0.0), world);
        let pos = pos2(pos.x, pos.y);
        match &mut data.phase {
            ShopPhase::Buy => {
                for entity in Self::all_offers(world) {
                    ShopOffer::draw_buy_panel(entity, world);
                }
                Window::new("reroll")
                    .fixed_pos(pos2(pos.x, pos.y))
                    .collapsible(false)
                    .title_bar(false)
                    .resizable(false)
                    .default_width(10.0)
                    .show(ctx, |ui| {
                        ui.set_enabled(Self::reroll_affordable(world));
                        ui.vertical_centered(|ui| {
                            let btn = Button::new(
                                RichText::new(format!("-{}g", Self::REROLL_PRICE))
                                    .size(20.0)
                                    .color(hex_color!("#00E5FF"))
                                    .text_style(egui::TextStyle::Button),
                            )
                            .min_size(egui::vec2(100.0, 0.0));
                            ui.label("Reroll");
                            if ui.add(btn).clicked() {
                                Self::buy_reroll(world).unwrap();
                            }
                        })
                    });
            }
            ShopPhase::Sacrifice { selected } => {
                for unit in UnitPlugin::collect_faction(Faction::Team, world) {
                    let slot = VarState::get(unit, world).get_int(VarName::Slot).unwrap() as usize;
                    entity_panel(unit, vec2(0.0, -1.5), None, "sacrifice", world).show(ctx, |ui| {
                        if ui.button("Sacrifice").clicked() {
                            selected.insert(slot);
                        }
                    });
                    if selected.contains(&slot) {
                        entity_panel(unit, default(), None, "cross", world).show(ctx, |ui| {
                            ui.label(RichText::new("X").color(hex_color!("#D50000")).size(80.0));
                        });
                    }
                }
                Area::new("accept sacrifice")
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_enabled(
                            Save::get(world).unwrap().team.units.len() - selected.len()
                                < SACRIFICE_SLOT,
                        );
                        if ui
                            .button(
                                RichText::new(format!("Accept Sacrifice +{} g", selected.len()))
                                    .color(hex_color!("#D50000"))
                                    .size(30.0)
                                    .text_style(egui::TextStyle::Button),
                            )
                            .clicked()
                        {
                            for entity in UnitPlugin::collect_faction(Faction::Team, world) {
                                if selected.contains(
                                    &(VarState::get(entity, world).get_int(VarName::Slot).unwrap()
                                        as usize),
                                ) {
                                    world.entity_mut(entity).despawn_recursive();
                                }
                            }
                            Self::change_g(selected.len() as i32, world).unwrap();
                            sacrifice_accepted = true;
                        }
                    });
            }
        }
        if sacrifice_accepted {
            UnitPlugin::fill_slot_gaps(Faction::Team, world);
            UnitPlugin::translate_to_slots(world);
            data.phase = ShopPhase::Buy;
        }
        if let Some(team_state) = PackedTeam::state(Faction::Team, world) {
            let g = team_state.get_int(VarName::G).unwrap_or_default();
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
        }
        Area::new("level number")
            .anchor(Align2::CENTER_TOP, [0.0, 20.0])
            .show(ctx, |ui| {
                let current_level = data.next_level_num;
                ui.label(
                    RichText::new(format!(
                        "Level {current_level} {}",
                        if Ladder::is_on_last_level(world) {
                            "(last)"
                        } else {
                            ""
                        }
                    ))
                    .size(40.0)
                    .color(hex_color!("#0091EA"))
                    .text_style(egui::TextStyle::Heading),
                );
            });
        Window::new("Next Enemy")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::RIGHT_CENTER, [-20.0, 0.0])
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let team = &data.next_team;
                    let unit = &team.units[0];

                    ui.heading(
                        RichText::new(format!("{} {}/{}", unit.name, unit.hp, unit.atk))
                            .size(20.0)
                            .color(hex_color!("#B71C1C")),
                    );

                    ui.label(RichText::new(format!("x{}", team.units.len())).size(25.0));
                    if !unit.statuses.is_empty() {
                        let (name, charges) = &unit.statuses[0];
                        ui.label(RichText::new(format!("with {name} ({charges})")));
                    }
                    ui.set_enabled(data.phase.eq(&ShopPhase::Buy));
                    let btn = Button::new(
                        RichText::new("Go")
                            .size(25.0)
                            .color(hex_color!("#B71C1C"))
                            .text_style(egui::TextStyle::Button),
                    )
                    .min_size(egui::vec2(100.0, 0.0));
                    if ui.add(btn).clicked() {
                        GameState::change(GameState::Battle, world);
                        GameTimer::get_mut(world).reset();
                    }
                });
            });
        world.insert_resource(data);
    }

    pub fn reroll_affordable(world: &mut World) -> bool {
        Self::get_g(world) >= Self::REROLL_PRICE
    }
    pub fn can_afford(price: i32, world: &mut World) -> bool {
        Self::get_g(world) >= price
    }

    pub fn buy_unit(unit: Entity, world: &mut World) -> Result<()> {
        let team = PackedTeam::entity(Faction::Team, world).unwrap();
        world
            .entity_mut(unit)
            .set_parent(team)
            .insert(ActiveTeam)
            .remove::<ShopOffer>();
        VarState::push_back(unit, VarName::Slot, Change::new(VarValue::Int(0)), world);
        UnitPlugin::fill_slot_gaps(Faction::Team, world);
        UnitPlugin::translate_to_slots(world);
        Self::change_g(-Self::UNIT_PRICE, world)
    }

    pub fn buy_reroll(world: &mut World) -> Result<()> {
        Self::clear_showcase(world);
        Self::fill_showcase(world);
        Self::change_g(-Self::REROLL_PRICE, world)
    }

    pub fn get_g(world: &mut World) -> i32 {
        PackedTeam::state(Faction::Team, world)
            .and_then(|s| s.get_int(VarName::G).ok())
            .unwrap_or_default()
    }

    pub fn change_g(delta: i32, world: &mut World) -> Result<()> {
        debug!("Change g {delta}");
        VarState::change_int(
            PackedTeam::entity(Faction::Team, world).unwrap(),
            VarName::G,
            delta,
            world,
        )
    }

    pub fn all_offers(world: &mut World) -> Vec<Entity> {
        world
            .query_filtered::<Entity, With<ShopOffer>>()
            .iter(world)
            .collect_vec()
    }
}

#[derive(Component, Clone, Debug)]
pub struct ShopOffer {
    pub name: String,
    pub description: String,
    pub price: i32,
    pub product: OfferProduct,
}

#[derive(Clone, Debug)]
pub enum OfferProduct {
    Unit,
    Status { name: String, charges: i32 },
}

impl OfferProduct {
    pub fn do_buy(&self, entity: Entity, world: &mut World) -> Result<()> {
        match self {
            OfferProduct::Unit => ShopPlugin::buy_unit(entity, world),
            OfferProduct::Status { name, charges } => {
                ActionPlugin::push_back_cluster(default(), world);
                for unit in UnitPlugin::collect_faction(Faction::Team, world)
                    .into_iter()
                    .rev()
                {
                    ActionPlugin::push_back(
                        Effect::AddStatus(name.clone()),
                        Context::from_target(unit, world)
                            .set_var(VarName::Charges, VarValue::Int(*charges))
                            .take(),
                        world,
                    );
                }
                ActionPlugin::spin(0.0, 0.2, world);
                world.entity_mut(entity).despawn_recursive();
                Ok(())
            }
        }
    }
}

impl ShopOffer {
    pub fn draw_buy_panel(entity: Entity, world: &mut World) {
        let so = world.get::<ShopOffer>(entity).unwrap().clone();
        let window = entity_panel(entity, vec2(0.0, -1.5), None, "buy_panel", world);
        let ctx = &egui_context(world);
        window.show(ctx, |ui: &mut egui::Ui| {
            ui.set_enabled(ShopPlugin::can_afford(so.price, world));
            ui.vertical_centered(|ui| {
                let btn = Button::new(
                    RichText::new(format!("-{}g", so.price))
                        .size(20.0)
                        .color(hex_color!("#00E5FF"))
                        .text_style(egui::TextStyle::Button),
                )
                .min_size(egui::vec2(100.0, 0.0));
                ui.label("Buy");
                if ui.add(btn).clicked() {
                    so.product.do_buy(entity, world).unwrap();
                }
            })
        });
        if !so.description.is_empty() {
            show_description_panels(entity, &so.description, world);
        }
    }
}

use super::*;

use bevy::input::common_conditions::input_just_pressed;
use rand::seq::IteratorRandom;

pub struct ShopPlugin;

#[derive(Resource, Clone)]
pub struct ShopData {
    pub next_team: PackedTeam,
    pub next_team_cards: Vec<(UnitCard, usize)>,
    pub next_level_num: usize,
    pub enemy_panel_expanded: bool,
    pub phase: ShopPhase,
}

const G_PER_ROUND: i32 = 4;
const HERO_COPIES_IN_POOL: usize = 5;
const UNIT_PRICE: i32 = 3;
const STATUS_PRICE: i32 = 2;
const REROLL_PRICE: i32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShopState {
    pub pool: Vec<ShopOffer>,
    pub case: Vec<(Entity, ShopOffer)>,
    pub g: i32,
}

impl ShopState {
    pub fn new(world: &mut World) -> Self {
        let mut result = Self {
            pool: Self::new_pool(world),
            case: default(),
            g: G_PER_ROUND,
        };
        result.fill_case(3, 2, world);
        result
    }
    fn can_afford(&self, g: i32) -> bool {
        self.g >= g
    }
    fn change_g(&mut self, delta: i32) {
        self.g += delta;
    }
    fn new_pool(world: &World) -> Vec<ShopOffer> {
        let heroes = &Pools::get(world).heroes;
        let total_heroes = heroes.len();
        Pools::get(world)
            .heroes
            .iter()
            .cycle()
            .take(HERO_COPIES_IN_POOL * total_heroes)
            .map(|(_, u)| ShopOffer {
                price: UNIT_PRICE,
                product: OfferProduct::Unit { unit: u.clone() },
                available: true,
            })
            .collect_vec()
    }
    fn refresh_case(&mut self, world: &mut World) {
        self.take_case(world);
        self.fill_case(3, 2, world);
    }
    fn take_case(&mut self, world: &mut World) {
        for (entity, mut offer) in self.case.drain(..) {
            if !offer.available {
                continue;
            }
            match &offer.product {
                OfferProduct::Unit { .. } => {
                    let unit = PackedUnit::pack(entity, world);
                    offer.product = OfferProduct::Unit { unit };
                    self.pool.push(offer);
                }
                OfferProduct::Status { .. } => {}
            }
            world.entity_mut(entity).despawn_recursive();
        }
    }
    fn fill_case(&mut self, heroes: usize, statuses: usize, world: &mut World) {
        let mut slot = 1;
        for _ in 0..heroes {
            if self.pool.is_empty() {
                self.pool = Self::new_pool(world);
            }
            let offer = self
                .pool
                .swap_remove((0..self.pool.len()).choose(&mut thread_rng()).unwrap());
            let entity = offer.product.spawn(slot, world);
            slot += 1;
            self.case.push((entity, offer));
        }
        UnitPlugin::translate_to_slots(world);
        for _ in 0..statuses {
            let status = Pools::get(world)
                .statuses
                .values()
                .filter(|s| s.shop_charges > 0)
                .choose(&mut thread_rng())
                .unwrap()
                .clone();
            let product = OfferProduct::Status {
                name: status.name,
                charges: status.shop_charges,
            };
            let entity = product.spawn(slot, world);
            slot += 1;
            self.case.push((
                entity,
                ShopOffer {
                    price: STATUS_PRICE,
                    product,
                    available: true,
                },
            ));
        }
    }
    fn buy(&mut self, entity: Entity, world: &mut World) -> Result<()> {
        let (_, offer) = self
            .case
            .iter_mut()
            .find(|(e, _)| entity.eq(e))
            .context("Failed to find offer")?;
        let price = offer.buy(entity, world)?;
        self.change_g(-price);
        Ok(())
    }
    fn respawn_case(&mut self, world: &mut World) {
        for (ind, (entity, offer)) in self.case.iter_mut().enumerate() {
            if let Some(entity) = world.get_entity_mut(*entity) {
                entity.despawn_recursive();
            }
            if offer.available {
                *entity = offer.product.spawn(ind + 1, world);
            }
        }
        UnitPlugin::translate_to_slots(world);
    }
}

#[derive(Clone, PartialEq)]
pub enum ShopPhase {
    Buy,
    Sacrifice { selected: HashSet<usize> },
}

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::on_enter)
            .add_systems(
                OnTransition {
                    from: GameState::Battle,
                    to: GameState::Shop,
                },
                Self::transition_from_battle.after(Self::on_enter),
            )
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
                )
                    .run_if(in_state(GameState::Shop)),),
            );
    }
}

impl ShopPlugin {
    fn win(world: &mut World) {
        Self::transition_to_battle(world);
        Save::get(world)
            .unwrap()
            .register_victory()
            .save(world)
            .unwrap();
        Self::on_enter(world);
    }

    fn on_enter(world: &mut World) {
        GameTimer::get_mut(world).reset();
        let mut save = Save::get(world).unwrap();
        if save.climb.shop.case.is_empty() {
            save.climb.shop.refresh_case(world);
        } else {
            save.climb.shop.respawn_case(world);
        }
        let team_len = save.climb.team.units.len();
        save.climb.team.clone().unpack(Faction::Team, world);
        save.save(world).unwrap();
        UnitPlugin::translate_to_slots(world);
        ActionPlugin::set_timeframe(0.05, world);
        let phase = match team_len < SACRIFICE_SLOT {
            true => ShopPhase::Buy,
            false => ShopPhase::Sacrifice {
                selected: default(),
            },
        };
        let (next_team, next_level_num) = Tower::load_current(world);
        let next_team_cards = next_team.get_cards(world);
        world.insert_resource(ShopData {
            next_team_cards,
            next_team,
            next_level_num: next_level_num + 1,
            phase,
            enemy_panel_expanded: false,
        });
    }

    fn transition_from_battle(world: &mut World) {
        let mut save = Save::get(world).unwrap();
        save.climb.shop.change_g(G_PER_ROUND);
        save.save(world).unwrap();
    }

    fn on_exit(world: &mut World) {
        Self::pack_active_team(world).unwrap();
        UnitPlugin::despawn_all_teams(world);
        Representation::despawn_all(world);
    }

    fn transition_to_battle(world: &mut World) {
        let left = Self::active_team(world).unwrap();
        let (right, ind) = Tower::load_current(world);
        BattlePlugin::load_teams(left, right, Some(ind), world);
    }

    fn input(world: &mut World) {
        if just_pressed(KeyCode::G, world) {
            Self::change_g(10, world).unwrap();
        }
        if just_pressed(KeyCode::S, world) {
            Save::store_current(world).unwrap();
        }
        if just_pressed(KeyCode::L, world) {
            Save::load_stored(world).unwrap();
            Self::on_enter(world);
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
        Ok(Save::get(world)?.climb.team)
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut data = world.remove_resource::<ShopData>().unwrap();
        let mut sacrifice_accepted = false;

        let pos = UnitPlugin::get_slot_position(Faction::Shop, 0) - vec2(1.0, 0.0);
        let pos = world_to_screen(pos.extend(0.0), world);
        let pos = pos2(pos.x, pos.y);
        let save = Save::get(world).unwrap();
        match &mut data.phase {
            ShopPhase::Buy => {
                ShopOffer::draw_buy_panels(world);
                Area::new("reroll").fixed_pos(pos).show(ctx, |ui| {
                    ui.set_width(120.0);
                    frame(ui, |ui| {
                        ui.set_enabled(save.climb.shop.can_afford(REROLL_PRICE));
                        ui.label("Reroll".add_color(white()).rich_text());
                        if ui
                            .button(
                                format!("-{}g", REROLL_PRICE)
                                    .add_color(yellow())
                                    .rich_text()
                                    .size(20.0),
                            )
                            .clicked()
                        {
                            Self::buy_reroll(world).unwrap();
                        }
                    });
                });
                Self::show_next_enemy_window(&mut data, world);
            }
            ShopPhase::Sacrifice { selected } => {
                for unit in UnitPlugin::collect_faction(Faction::Team, world) {
                    let slot = VarState::get(unit, world).get_int(VarName::Slot).unwrap() as usize;
                    window("sacrifice")
                        .id(unit)
                        .set_width(80.0)
                        .resizable(false)
                        .title_bar(false)
                        .stroke(false)
                        .entity_anchor(unit, Align2::CENTER_TOP, vec2(0.0, 70.0), world)
                        .show(ctx, |ui| {
                            frame(ui, |ui| {
                                ui.set_width(100.0);
                                let is_selected = selected.contains(&slot);
                                let text = "SACRIFICE";
                                if if is_selected {
                                    ui.button_primary(text)
                                } else {
                                    ui.button(text)
                                }
                                .clicked()
                                {
                                    if is_selected {
                                        selected.remove(&slot);
                                    } else {
                                        selected.insert(slot);
                                    }
                                }
                            });
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
                            save.climb.team.units.len() - selected.len() < SACRIFICE_SLOT,
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
                            Self::pack_active_team(world).unwrap();
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

        let g = save.climb.shop.g;
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
        world.insert_resource(data);
    }

    fn show_next_enemy_window(data: &mut ShopData, world: &mut World) {
        let len = data.next_team_cards.len();
        window("NEXT ENEMY")
            .anchor(Align2::RIGHT_CENTER, [-10.0, 0.0])
            .show(&egui_context(world), |ui| {
                Frame::none().inner_margin(8.0).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            format!(
                                "Level {}/{}",
                                data.next_level_num,
                                Tower::total_levels(world)
                            )
                            .add_color(white())
                            .rich_text(),
                        );
                        let mut can_expand = false;
                        ui.columns(len, |ui| {
                            for (ind, (card, count)) in data.next_team_cards.iter().enumerate() {
                                can_expand |=
                                    !card.description.is_empty() || !card.statuses.is_empty();
                                ui[ind].vertical_centered(|ui| {
                                    if data.enemy_panel_expanded {
                                        card.show_frames(ui);
                                    } else {
                                        card.show_name(false, ui);
                                    }
                                    if *count > 1 {
                                        ui.heading(
                                            format!("x{count}").add_color(white()).rich_text(),
                                        );
                                    }
                                });
                            }
                        });

                        if can_expand
                            && if data.enemy_panel_expanded {
                                ui.button_primary("EXPAND")
                            } else {
                                ui.button("EXPAND")
                            }
                            .clicked()
                        {
                            data.enemy_panel_expanded = !data.enemy_panel_expanded;
                        }
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button_primary(RichText::new("GO").heading()).clicked() {
                                Self::go_to_battle(world);
                            }
                        });
                    });
                });
            });
    }

    fn go_to_battle(world: &mut World) {
        let mut save = Save::get(world).unwrap();
        save.climb.shop.take_case(world);
        save.save(world).unwrap();
        GameState::change(GameState::Battle, world);
    }

    pub fn buy_reroll(world: &mut World) -> Result<()> {
        let mut save = Save::get(world)?;
        if !save.climb.shop.can_afford(REROLL_PRICE) {
            return Err(anyhow!("Not enough g"));
        }
        save.climb.shop.refresh_case(world);
        save.climb.shop.change_g(-REROLL_PRICE);
        save.save(world)
    }

    pub fn get_g(world: &mut World) -> i32 {
        Save::get(world).unwrap().climb.shop.g
    }

    pub fn change_g(delta: i32, world: &mut World) -> Result<()> {
        debug!("Change g {delta}");
        let mut save = Save::get(world)?;
        save.climb.shop.g += delta;
        save.save(world)?;
        VarState::change_int(Faction::Team.team_entity(world), VarName::G, delta, world)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShopOffer {
    pub price: i32,
    pub available: bool,
    pub product: OfferProduct,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OfferProduct {
    Unit { unit: PackedUnit },
    Status { name: String, charges: i32 },
}

impl OfferProduct {
    pub fn spawn(&self, slot: usize, world: &mut World) -> Entity {
        let parent = Faction::Shop.team_entity(world);
        match self {
            OfferProduct::Unit { unit } => unit.clone().unpack(parent, Some(slot), world),
            OfferProduct::Status { name, charges } => {
                let status = Pools::get(world).statuses.get(name).unwrap();
                let entity = status.clone().unpack(parent, world);
                let pos = UnitPlugin::get_slot_position(Faction::Shop, slot);
                VarState::get_mut(entity, world)
                    .init(VarName::Position, VarValue::Vec2(pos))
                    .init(VarName::Charges, VarValue::Int(*charges));
                entity
            }
        }
    }
}

impl ShopOffer {
    fn buy(&mut self, entity: Entity, world: &mut World) -> Result<i32> {
        if !Save::get(world)?.climb.shop.can_afford(self.price) {
            return Err(anyhow!("Can't afford {}", self.price));
        }
        if !self.available {
            return Err(anyhow!("Offer is no longer available"));
        }
        self.available = false;
        match &self.product {
            OfferProduct::Unit { .. } => {
                let pos = VarState::get(entity, world).get_vec2(VarName::Position)?;
                let entity = PackedUnit::pack(entity, world).clone().unpack(
                    Faction::Team.team_entity(world),
                    None,
                    world,
                );
                VarState::get_mut(entity, world).init(VarName::Position, VarValue::Vec2(pos));
                UnitPlugin::fill_slot_gaps(Faction::Team, world);
                UnitPlugin::translate_to_slots(world);
            }
            OfferProduct::Status { name, charges } => {
                for unit in UnitPlugin::collect_faction(Faction::Team, world)
                    .into_iter()
                    .rev()
                {
                    let context = Context::from_target(unit, world)
                        .set_var(VarName::Charges, VarValue::Int(*charges))
                        .take();
                    ActionCluster::current(world)
                        .push_action_back(Effect::AddStatus(name.clone()), context);
                }
            }
        }
        world.entity_mut(entity).despawn_recursive();
        Ok(self.price)
    }

    fn draw_buy_panels(world: &mut World) {
        let ctx = &egui_context(world);
        let save = &mut Save::get(world).unwrap();
        for (entity, offer) in save.climb.shop.case.clone().iter_mut() {
            if !offer.available {
                continue;
            }
            match &offer.product {
                OfferProduct::Unit { .. } => {}
                OfferProduct::Status { name, charges } => {
                    window("BUY STATUS")
                        .id(&entity)
                        .title_bar(false)
                        .resizable(false)
                        .set_width(150.0)
                        .entity_anchor(*entity, Align2::CENTER_BOTTOM, vec2(0.0, -80.0), world)
                        .show(ctx, |ui| {
                            frame(ui, |ui| {
                                ui.vertical(|ui| {
                                    let color: Color32 = Pools::get_status_house(name, world)
                                        .unwrap()
                                        .color
                                        .clone()
                                        .into();
                                    ui.label(name.add_color(color).rich_text());
                                    let description = Pools::get_status(name, world)
                                        .unwrap()
                                        .description
                                        .to_colored()
                                        .inject_vars(&VarState::new_with(
                                            VarName::Charges,
                                            VarValue::Int(*charges),
                                        ));
                                    ui.label(description.widget());
                                });
                            });
                        });
                }
            }
            window("BUY")
                .id(&entity)
                .set_width(80.0)
                .resizable(false)
                .title_bar(false)
                .stroke(false)
                .entity_anchor(*entity, Align2::CENTER_TOP, vec2(0.0, 70.0), world)
                .show(ctx, |ui| {
                    ui.set_enabled(offer.available && save.climb.shop.can_afford(offer.price));
                    frame(ui, |ui| {
                        ui.set_width(100.0);
                        if ui
                            .button(
                                format!("-{} g", offer.price)
                                    .add_color(yellow())
                                    .rich_text()
                                    .size(20.0),
                            )
                            .clicked()
                        {
                            if let Err(e) = save.climb.shop.buy(*entity, world) {
                                error!("Buy error: {}", e);
                            } else {
                                save.save(world).unwrap();
                                ShopPlugin::pack_active_team(world).unwrap();
                            }
                        }
                    });
                });
        }
    }
}

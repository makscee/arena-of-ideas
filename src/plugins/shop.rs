use super::*;

use bevy_egui::egui::{self, Align2};
use bevy_egui::egui::{pos2, Button, Color32, RichText, Window};
use bevy_pkv::SetError;
use rand::seq::IteratorRandom;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::enter_state)
            .add_systems(OnExit(GameState::Shop), Self::leave_state)
            .add_systems(PostUpdate, Self::input)
            .add_systems(Update, Self::ui.run_if(in_state(GameState::Shop)));
    }
}

impl ShopPlugin {
    pub const UNIT_PRICE: i32 = 3;
    pub const REROLL_PRICE: i32 = 1;

    fn enter_state(world: &mut World) {
        Self::unpack_active_team(Faction::Team, world);
        UnitPlugin::translate_to_slots(world);
        Self::fill_showcase(world);
        Self::change_g(10, world).unwrap();
    }

    fn leave_state(world: &mut World) {
        Self::pack_active_team(world).unwrap();
        UnitPlugin::despawn_all(world);
        Self::clear_showcase(world);
    }

    fn input(world: &mut World) {
        if just_pressed(KeyCode::P, world) {
            Self::pack_active_team(world).unwrap();
            UnitPlugin::despawn_all(world);
            Self::unpack_active_team(Faction::Team, world);
        }
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
            let status = Pools::get_status("Strength", world).clone();
            let name = status.name.to_owned();
            let description = status.description.to_owned();
            let charges = status.state.get_int(VarName::Charges).unwrap_or(1);
            let entity = status.unpack(None, world).unwrap();
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

    pub fn pack_active_team(world: &mut World) -> Result<(), SetError> {
        let team = PackedTeam::pack(Faction::Team, world);
        debug!("Active team saved: {team:#?}");
        Save::get(world)
            .unwrap_or_default()
            .set_team(team)
            .save(world)
    }

    pub fn unpack_active_team(faction: Faction, world: &mut World) {
        if let Ok(save) = Save::get(world) {
            debug!("Unpacking saved team {:#?}", save.team);
            save.team.unpack(faction, world);
        } else {
            PackedTeam::spawn(faction, world);
        }
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        for entity in Self::all_offers(world) {
            ShopOffer::draw_buy_panel(entity, world);
        }
        if let Some(team_state) = PackedTeam::state(Faction::Team, world) {
            let g = team_state.get_int(VarName::G).unwrap_or_default();
            Window::new("Stats").show(&ctx, |ui| {
                ui.label(RichText::new(format!("G: {g}")).color(Color32::KHAKI));
            });
        }
        let pos = UnitPlugin::get_slot_position(Faction::Shop, 0);
        let pos = vec3(pos.x, pos.y, 0.0);
        let pos = world_to_screen(pos, world);
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
        Window::new("battle")
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                if ui.button("Go").clicked() {
                    world
                        .get_resource_mut::<NextState<GameState>>()
                        .unwrap()
                        .set(GameState::Battle);
                    GameTimer::get_mut(world).clear_save();
                    GameTimer::get_mut(world).reset();
                }
            });
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
                for unit in UnitPlugin::collect_faction(Faction::Team, world) {
                    Status::change_charges(name, unit, *charges, world).unwrap();
                }
                world.entity_mut(entity).despawn_recursive();
                Ok(())
            }
        }
    }
}

impl ShopOffer {
    pub fn draw_buy_panel(entity: Entity, world: &mut World) {
        let so = world.get::<ShopOffer>(entity).unwrap().clone();
        let window = entity_panel(entity, vec2(0.0, -1.5), "buy_panel", world);
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
            let window = entity_panel(entity, vec2(0.0, 1.1), "desc_panel", world);
            window.show(ctx, |ui| {
                ui.label(so.description);
            });
        }
    }
}

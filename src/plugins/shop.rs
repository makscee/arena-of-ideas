use super::*;

use bevy_egui::egui;
use bevy_egui::egui::{pos2, Button, Color32, RichText, Window};
use rand::seq::SliceRandom;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTeam>()
            .add_systems(OnEnter(GameState::Shop), Self::enter_state)
            .add_systems(PostUpdate, Self::input)
            .add_systems(Update, Self::ui);
    }
}

impl ShopPlugin {
    pub const UNIT_PRICE: i32 = 3;
    pub const REROLL_PRICE: i32 = 1;

    fn enter_state(world: &mut World) {
        if let Some(team) = &world.resource::<ActiveTeam>().team {
            team.clone().unpack(Faction::Team, world);
        } else {
            PackedTeam::spawn(Faction::Team, world);
        }
        Self::fill_showcase(world);
    }

    fn leave_state(world: &mut World) {}

    fn input(world: &mut World) {
        if just_pressed(KeyCode::P, world) {
            Self::pack_active_team(world);
            UnitPlugin::despawn_all(world);
            Self::unpack_active_team(world);
        }
        if just_pressed(KeyCode::C, world) {
            Self::change_g(10, world).unwrap();
        }
    }

    fn fill_showcase(world: &mut World) {
        let mut units = Vec::default();
        let pool = Pools::heroes(world).into_values().collect_vec();
        for _ in 0..5 {
            let unit = (*pool.choose(&mut rand::thread_rng()).unwrap()).clone();
            units.push(unit);
        }
        let team = PackedTeam::spawn(Faction::Shop, world).id();
        for unit in units {
            unit.unpack(team, None, world);
        }
        UnitPlugin::fill_slot_gaps(Faction::Shop, world);
        UnitPlugin::translate_to_slots(world);
    }

    fn clear_showcase(world: &mut World) {
        PackedTeam::despawn(Faction::Shop, world);
    }

    pub fn pack_active_team(world: &mut World) {
        let team = PackedTeam::pack(Faction::Team, world);
        let mut active_team = world.get_resource_mut::<ActiveTeam>().unwrap();
        active_team.team = Some(team);
    }

    pub fn unpack_active_team(world: &mut World) {
        world
            .get_resource::<ActiveTeam>()
            .unwrap()
            .team
            .clone()
            .expect("Tried to unpack emtpy Active Team")
            .unpack(Faction::Team, world);
        UnitPlugin::translate_to_slots(world);
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        for unit in UnitPlugin::collect_faction(Faction::Shop, world) {
            let window = UnitPlugin::draw_unit_panel(unit, vec2(0.0, -1.5), world);
            window.show(&ctx, |ui| {
                ui.set_enabled(Self::unit_affordable(world));
                ui.vertical_centered(|ui| {
                    let btn = Button::new(
                        RichText::new(format!("-{}g", Self::UNIT_PRICE))
                            .size(20.0)
                            .color(hex_color!("#00E5FF"))
                            .text_style(egui::TextStyle::Button),
                    )
                    .min_size(egui::vec2(100.0, 0.0));
                    ui.label("Buy");
                    if ui.add(btn).clicked() {
                        Self::buy_unit(unit, world).unwrap();
                    }
                })
            });
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
    }

    pub fn unit_affordable(world: &mut World) -> bool {
        Self::get_g(world) >= Self::UNIT_PRICE
    }
    pub fn reroll_affordable(world: &mut World) -> bool {
        Self::get_g(world) >= Self::REROLL_PRICE
    }

    pub fn buy_unit(unit: Entity, world: &mut World) -> Result<()> {
        let team = PackedTeam::entity(Faction::Team, world).unwrap();
        world.entity_mut(unit).set_parent(team);
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
}

#[derive(Resource, Default)]
pub struct ActiveTeam {
    pub team: Option<PackedTeam>,
}

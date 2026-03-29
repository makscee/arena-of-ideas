use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::plugins::ui::{rating_color, tier_color};
use crate::resources::game_state::GameState;

pub struct CollectionPlugin;

impl Plugin for CollectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollectionState>()
            .init_resource::<GameContent>()
            .add_systems(OnEnter(GameState::Home), init_mock_content)
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                collection_ui.run_if(in_state(GameState::Home)),
            );
    }
}

#[derive(Resource, Default)]
struct CollectionState {
    tab: CollectionTab,
    selected_ability: Option<usize>,
    selected_unit: Option<usize>,
}

#[derive(Default, PartialEq)]
enum CollectionTab {
    #[default]
    Abilities,
    Units,
}

/// Content data — will be populated from SpacetimeDB subscriptions later.
/// For now, uses mock data.
#[derive(Resource, Default)]
pub struct GameContent {
    pub abilities: Vec<AbilityData>,
    pub units: Vec<UnitData>,
}

#[derive(Clone)]
pub struct AbilityData {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub target_type: String,
    pub effect_script: String,
    pub parent_a: u64,
    pub parent_b: u64,
    pub rating: i32,
    pub status: String,
}

#[derive(Clone)]
pub struct UnitData {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub hp: i32,
    pub pwr: i32,
    pub tier: u8,
    pub trigger: String,
    pub ability_names: Vec<String>,
    pub rating: i32,
    pub status: String,
}

fn init_mock_content(mut content: ResMut<GameContent>) {
    if !content.abilities.is_empty() {
        return;
    }

    content.abilities = vec![
        AbilityData {
            id: 1,
            name: "Strike".into(),
            description: "Deals damage to a random enemy".into(),
            target_type: "Random Enemy".into(),
            effect_script: "ability_actions.deal_damage(target, X * level);".into(),
            parent_a: 0,
            parent_b: 0,
            rating: 12,
            status: "Active".into(),
        },
        AbilityData {
            id: 2,
            name: "Guard".into(),
            description: "Grants a shield to self".into(),
            target_type: "Self".into(),
            effect_script: "ability_actions.add_shield(owner, X * level);".into(),
            parent_a: 0,
            parent_b: 0,
            rating: 8,
            status: "Active".into(),
        },
        AbilityData {
            id: 3,
            name: "Heal".into(),
            description: "Restores health to a random ally".into(),
            target_type: "Random Ally".into(),
            effect_script: "ability_actions.heal_damage(target, X * level);".into(),
            parent_a: 0,
            parent_b: 0,
            rating: 15,
            status: "Active".into(),
        },
        AbilityData {
            id: 4,
            name: "Curse".into(),
            description: "Reduces a random enemy's power".into(),
            target_type: "Random Enemy".into(),
            effect_script: "ability_actions.change_stat(target, \"pwr\", -level);".into(),
            parent_a: 0,
            parent_b: 0,
            rating: -2,
            status: "Active".into(),
        },
    ];

    content.units = vec![
        UnitData {
            id: 1,
            name: "Footsoldier".into(),
            description: "A basic melee fighter".into(),
            hp: 3,
            pwr: 2,
            tier: 1,
            trigger: "Before Strike".into(),
            ability_names: vec!["Strike".into()],
            rating: 5,
            status: "Active".into(),
        },
        UnitData {
            id: 2,
            name: "Shieldbearer".into(),
            description: "A defensive unit that protects itself".into(),
            hp: 4,
            pwr: 1,
            tier: 1,
            trigger: "Battle Start".into(),
            ability_names: vec!["Guard".into()],
            rating: 10,
            status: "Active".into(),
        },
        UnitData {
            id: 3,
            name: "Medic".into(),
            description: "Heals allies at the end of each turn".into(),
            hp: 2,
            pwr: 3,
            tier: 1,
            trigger: "Turn End".into(),
            ability_names: vec!["Heal".into()],
            rating: 18,
            status: "Active".into(),
        },
        UnitData {
            id: 4,
            name: "Knight".into(),
            description: "A powerful striker with high health".into(),
            hp: 6,
            pwr: 4,
            tier: 2,
            trigger: "Before Strike".into(),
            ability_names: vec!["Strike".into()],
            rating: 7,
            status: "Active".into(),
        },
        UnitData {
            id: 5,
            name: "Paladin".into(),
            description: "A holy warrior that strikes and guards".into(),
            hp: 8,
            pwr: 6,
            tier: 3,
            trigger: "Before Strike".into(),
            ability_names: vec!["Strike".into(), "Guard".into()],
            rating: 22,
            status: "Active".into(),
        },
        UnitData {
            id: 6,
            name: "Warlock".into(),
            description: "A dark caster that strikes and curses".into(),
            hp: 7,
            pwr: 7,
            tier: 3,
            trigger: "Turn End".into(),
            ability_names: vec!["Strike".into(), "Curse".into()],
            rating: -1,
            status: "Incubator".into(),
        },
    ];
}

fn collection_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<CollectionState>,
    content: Res<GameContent>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Arena of Ideas");
            ui.separator();
            if ui
                .selectable_label(state.tab == CollectionTab::Abilities, "Abilities")
                .clicked()
            {
                state.tab = CollectionTab::Abilities;
            }
            if ui
                .selectable_label(state.tab == CollectionTab::Units, "Units")
                .clicked()
            {
                state.tab = CollectionTab::Units;
            }
            ui.separator();
            if ui.button("Create").clicked() {
                next_game_state.set(GameState::Create);
            }
            if ui.button("Incubator").clicked() {
                next_game_state.set(GameState::Incubator);
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| match state.tab {
        CollectionTab::Abilities => {
            abilities_panel(ui, &content, &mut state);
        }
        CollectionTab::Units => {
            units_panel(ui, &content, &mut state);
        }
    });
}

fn abilities_panel(ui: &mut egui::Ui, content: &GameContent, state: &mut CollectionState) {
    ui.heading("Abilities");
    ui.separator();

    if content.abilities.is_empty() {
        ui.label("No abilities loaded yet. Connect to server to see content.");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, ability) in content.abilities.iter().enumerate() {
            let selected = state.selected_ability == Some(i);
            let response = ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(crate::plugins::ui::colors::ABILITY_COLOR, &ability.name);
                    ui.label("→");
                    ui.label(&ability.target_type);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(
                            rating_color(ability.rating),
                            format!("{:+}", ability.rating),
                        );
                        ui.label(&ability.status);
                    });
                });

                if selected {
                    ui.separator();
                    ui.label(&ability.description);
                    ui.separator();
                    ui.label("Script:");
                    ui.code(&ability.effect_script);
                }
            });

            if response.response.clicked() {
                state.selected_ability = if selected { None } else { Some(i) };
            }
        }
    });
}

fn units_panel(ui: &mut egui::Ui, content: &GameContent, state: &mut CollectionState) {
    ui.heading("Units");
    ui.separator();

    if content.units.is_empty() {
        ui.label("No units loaded yet. Connect to server to see content.");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, unit) in content.units.iter().enumerate() {
            let selected = state.selected_unit == Some(i);
            let response = ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(tier_color(unit.tier), format!("T{}", unit.tier));
                    ui.colored_label(crate::plugins::ui::colors::UNIT_COLOR, &unit.name);
                    ui.label(format!("{}hp / {}pwr", unit.hp, unit.pwr));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(rating_color(unit.rating), format!("{:+}", unit.rating));
                        ui.label(&unit.status);
                    });
                });

                if selected {
                    ui.separator();
                    ui.label(&unit.description);
                    ui.horizontal(|ui| {
                        ui.label("Trigger:");
                        ui.strong(&unit.trigger);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Abilities:");
                        for name in &unit.ability_names {
                            ui.colored_label(crate::plugins::ui::colors::ABILITY_COLOR, name);
                        }
                    });
                }
            });

            if response.response.clicked() {
                state.selected_unit = if selected { None } else { Some(i) };
            }
        }
    });
}

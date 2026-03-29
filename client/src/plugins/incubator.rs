use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::module_bindings::*;
use crate::plugins::collection::GameContent;
use crate::plugins::connect::StdbConnection;
use crate::plugins::ui::{colors, rating_color, tier_color};
use crate::resources::game_state::GameState;

pub struct IncubatorPlugin;

impl Plugin for IncubatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IncubatorState>().add_systems(
            bevy_egui::EguiPrimaryContextPass,
            incubator_ui.run_if(in_state(GameState::Incubator)),
        );
    }
}

#[derive(Resource, Default)]
struct IncubatorState {
    tab: IncubatorTab,
    sort_by_rating: bool,
}

#[derive(Default, PartialEq)]
enum IncubatorTab {
    #[default]
    Abilities,
    Units,
    EvolutionTree,
}

fn call_vote(stdb: &StdbConnection, entity_kind: &str, entity_id: u64, value: i8) {
    if let Some(ref conn) = stdb.conn {
        if let Err(e) = conn
            .reducers
            .vote_cast(entity_kind.to_string(), entity_id, value)
        {
            warn!("Vote failed: {:?}", e);
        }
    }
}

fn incubator_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<IncubatorState>,
    content: Res<GameContent>,
    stdb: Res<StdbConnection>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::top("incubator_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                next_game_state.set(GameState::Home);
            }
            ui.separator();
            ui.heading("Incubator");
            ui.separator();
            if ui
                .selectable_label(state.tab == IncubatorTab::Abilities, "Abilities")
                .clicked()
            {
                state.tab = IncubatorTab::Abilities;
            }
            if ui
                .selectable_label(state.tab == IncubatorTab::Units, "Units")
                .clicked()
            {
                state.tab = IncubatorTab::Units;
            }
            if ui
                .selectable_label(state.tab == IncubatorTab::EvolutionTree, "Evolution Tree")
                .clicked()
            {
                state.tab = IncubatorTab::EvolutionTree;
            }
            ui.separator();
            ui.checkbox(&mut state.sort_by_rating, "Sort by rating");
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| match state.tab {
        IncubatorTab::Abilities => incubator_abilities(ui, &content, &state, &stdb),
        IncubatorTab::Units => incubator_units(ui, &content, &state, &stdb),
        IncubatorTab::EvolutionTree => evolution_tree(ui, &content),
    });
}

fn incubator_abilities(
    ui: &mut egui::Ui,
    content: &GameContent,
    state: &IncubatorState,
    stdb: &StdbConnection,
) {
    ui.heading("Ability Incubator");
    ui.label("Vote on abilities to decide what enters the game.");
    ui.separator();

    let mut abilities: Vec<_> = content.abilities.iter().enumerate().collect();
    if state.sort_by_rating {
        abilities.sort_by(|a, b| b.1.rating.cmp(&a.1.rating));
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (_i, ability) in &abilities {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(colors::ABILITY_COLOR, &ability.name);
                    ui.label("→");
                    ui.label(&ability.target_type);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("▼").clicked() {
                            call_vote(stdb, "ability", ability.id, -1);
                        }
                        ui.colored_label(
                            rating_color(ability.rating),
                            format!("{:+}", ability.rating),
                        );
                        if ui.small_button("▲").clicked() {
                            call_vote(stdb, "ability", ability.id, 1);
                        }
                        ui.label(&ability.status);
                    });
                });
                ui.label(&ability.description);
            });
        }
    });
}

fn incubator_units(
    ui: &mut egui::Ui,
    content: &GameContent,
    state: &IncubatorState,
    stdb: &StdbConnection,
) {
    ui.heading("Unit Incubator");
    ui.label("Vote on units to decide what enters the game.");
    ui.separator();

    let mut units: Vec<_> = content.units.iter().enumerate().collect();
    if state.sort_by_rating {
        units.sort_by(|a, b| b.1.rating.cmp(&a.1.rating));
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (_i, unit) in &units {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(tier_color(unit.tier), format!("T{}", unit.tier));
                    ui.colored_label(colors::UNIT_COLOR, &unit.name);
                    ui.label(format!("{}hp / {}pwr", unit.hp, unit.pwr));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("▼").clicked() {
                            call_vote(stdb, "unit", unit.id, -1);
                        }
                        ui.colored_label(rating_color(unit.rating), format!("{:+}", unit.rating));
                        if ui.small_button("▲").clicked() {
                            call_vote(stdb, "unit", unit.id, 1);
                        }
                        ui.label(&unit.status);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Trigger:");
                    ui.strong(&unit.trigger);
                    ui.label("Abilities:");
                    for name in &unit.ability_names {
                        ui.colored_label(colors::ABILITY_COLOR, name);
                    }
                });
            });
        }
    });
}

fn evolution_tree(ui: &mut egui::Ui, content: &GameContent) {
    ui.heading("Ability Evolution Tree");
    ui.label("All abilities trace back to the primordial set.");
    ui.separator();

    if content.abilities.is_empty() {
        ui.label("No abilities loaded.");
        return;
    }

    // Build name lookup
    let name_of = |id: u64| -> String {
        content
            .abilities
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| format!("#{}", id))
    };

    // Split into primordial (no parents) and bred (has parents)
    let primordial: Vec<_> = content
        .abilities
        .iter()
        .filter(|a| a.parent_a == 0 && a.parent_b == 0)
        .collect();
    let bred: Vec<_> = content
        .abilities
        .iter()
        .filter(|a| a.parent_a != 0 || a.parent_b != 0)
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Primordial abilities
        ui.heading("Primordial Abilities");
        for ability in &primordial {
            ui.horizontal(|ui| {
                ui.colored_label(colors::ABILITY_COLOR, &ability.name);
                ui.label("—");
                ui.label(&ability.description);
                ui.colored_label(
                    rating_color(ability.rating),
                    format!("{:+}", ability.rating),
                );
            });
        }

        if !bred.is_empty() {
            ui.separator();
            ui.heading("Bred Abilities");
            for ability in &bred {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(colors::ABILITY_COLOR, &ability.name);
                        ui.colored_label(
                            rating_color(ability.rating),
                            format!("{:+}", ability.rating),
                        );
                    });
                    ui.label(&ability.description);
                    ui.horizontal(|ui| {
                        ui.label("Parents:");
                        ui.colored_label(colors::ABILITY_COLOR, name_of(ability.parent_a));
                        ui.label("×");
                        ui.colored_label(colors::ABILITY_COLOR, name_of(ability.parent_b));
                    });
                });
            }
        } else {
            ui.separator();
            ui.label("No bred abilities yet. Use Create → Breed Ability to start evolving!");
        }
    });
}

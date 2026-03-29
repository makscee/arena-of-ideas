use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::module_bindings::*;
use crate::plugins::collection::GameContent;
use crate::plugins::connect::StdbConnection;
use crate::plugins::ui::{colors, tier_color};
use crate::resources::game_state::GameState;

pub struct CreatePlugin;

impl Plugin for CreatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CreateState>()
            .add_systems(Update, create_ui.run_if(in_state(GameState::Create)));
    }
}

#[derive(Resource, Default)]
pub struct CreateState {
    pub tab: CreateTab,
    // Ability breeding
    pub selected_parent_a: Option<usize>,
    pub selected_parent_b: Option<usize>,
    pub breeding_prompt: String,
    pub breeding_result: Option<BreedingResult>,
    pub breeding_status: RequestStatus,
    // Unit assembly
    pub unit_trigger_idx: usize,
    pub unit_tier: u8,
    pub unit_selected_abilities: Vec<usize>,
    pub unit_prompt: String,
    pub unit_hp: i32,
    pub unit_pwr: i32,
    pub unit_result: Option<UnitResult>,
    pub unit_status: RequestStatus,
}

#[derive(Default, PartialEq)]
pub enum CreateTab {
    #[default]
    BreedAbility,
    AssembleUnit,
}

#[derive(Default, PartialEq)]
pub enum RequestStatus {
    #[default]
    Idle,
    Pending,
    Done,
    Error(String),
}

#[derive(Clone)]
pub struct BreedingResult {
    pub name: String,
    pub description: String,
    pub target_type: String,
    pub effect_script: String,
    pub explanation: String,
}

#[derive(Clone)]
pub struct UnitResult {
    pub name: String,
    pub painter_script: String,
}

const TRIGGERS: &[&str] = &[
    "Battle Start",
    "Turn End",
    "Before Death",
    "Ally Death",
    "Before Strike",
    "After Strike",
    "Damage Taken",
    "Damage Dealt",
];

fn create_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<CreateState>,
    content: Res<GameContent>,
    stdb: Res<StdbConnection>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::top("create_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                next_game_state.set(GameState::Home);
            }
            ui.separator();
            ui.heading("Create");
            ui.separator();
            if ui
                .selectable_label(state.tab == CreateTab::BreedAbility, "Breed Ability")
                .clicked()
            {
                state.tab = CreateTab::BreedAbility;
            }
            if ui
                .selectable_label(state.tab == CreateTab::AssembleUnit, "Assemble Unit")
                .clicked()
            {
                state.tab = CreateTab::AssembleUnit;
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| match state.tab {
        CreateTab::BreedAbility => breed_ability_ui(ui, &mut state, &content, &stdb),
        CreateTab::AssembleUnit => assemble_unit_ui(ui, &mut state, &content, &stdb),
    });
}

fn breed_ability_ui(
    ui: &mut egui::Ui,
    state: &mut ResMut<CreateState>,
    content: &GameContent,
    stdb: &StdbConnection,
) {
    ui.heading("Breed New Ability");
    ui.label("Pick two parent abilities and describe how to combine them.");
    ui.separator();

    ui.columns(2, |columns| {
        columns[0].label("Parent A:");
        for (i, ability) in content.abilities.iter().enumerate() {
            let selected = state.selected_parent_a == Some(i);
            if columns[0]
                .selectable_label(selected, &ability.name)
                .clicked()
            {
                state.selected_parent_a = Some(i);
            }
        }

        columns[1].label("Parent B:");
        for (i, ability) in content.abilities.iter().enumerate() {
            let selected = state.selected_parent_b == Some(i);
            if columns[1]
                .selectable_label(selected, &ability.name)
                .clicked()
            {
                state.selected_parent_b = Some(i);
            }
        }
    });

    ui.separator();

    if let (Some(a), Some(b)) = (state.selected_parent_a, state.selected_parent_b) {
        if a == b {
            ui.colored_label(egui::Color32::RED, "Cannot breed an ability with itself!");
        } else if let (Some(pa), Some(pb)) = (content.abilities.get(a), content.abilities.get(b)) {
            ui.horizontal(|ui| {
                ui.colored_label(colors::ABILITY_COLOR, &pa.name);
                ui.label("×");
                ui.colored_label(colors::ABILITY_COLOR, &pb.name);
            });

            ui.separator();
            ui.label("How should these combine?");
            ui.text_edit_multiline(&mut state.breeding_prompt);

            let can_submit =
                !state.breeding_prompt.is_empty() && state.breeding_status == RequestStatus::Idle;

            if ui
                .add_enabled(can_submit, egui::Button::new("Breed with AI"))
                .clicked()
            {
                // Call the gen_breed_ability reducer
                if let Some(ref conn) = stdb.conn {
                    match conn.reducers.gen_breed_ability(
                        pa.id,
                        pb.id,
                        state.breeding_prompt.clone(),
                    ) {
                        Ok(_) => {
                            state.breeding_status = RequestStatus::Pending;
                            info!("Breeding request sent: {} × {}", pa.name, pb.name);
                        }
                        Err(e) => {
                            state.breeding_status = RequestStatus::Error(format!("{:?}", e));
                        }
                    }
                } else {
                    state.breeding_status =
                        RequestStatus::Error("Not connected to server".to_string());
                }
            }

            match &state.breeding_status {
                RequestStatus::Pending => {
                    ui.spinner();
                    ui.label("AI generation request submitted. Waiting for result...");
                }
                RequestStatus::Error(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
                    if ui.button("Dismiss").clicked() {
                        state.breeding_status = RequestStatus::Idle;
                    }
                }
                _ => {}
            }
        }
    }

    // Show result
    let result_clone = state.breeding_result.clone();
    if let Some(result) = &result_clone {
        ui.separator();
        ui.heading("Result");
        ui.group(|ui| {
            ui.colored_label(colors::ABILITY_COLOR, &result.name);
            ui.label(&result.description);
            ui.horizontal(|ui| {
                ui.label("Target:");
                ui.strong(&result.target_type);
            });
            ui.separator();
            ui.label("Script:");
            ui.code(&result.effect_script);
            ui.separator();
            ui.label("AI reasoning:");
            ui.label(&result.explanation);
        });

        ui.horizontal(|ui| {
            if ui.button("Accept → Incubator").clicked() {
                if let Some(ref conn) = stdb.conn {
                    let target_type = match result.target_type.as_str() {
                        "RandomEnemy" => TargetType::RandomEnemy,
                        "AllEnemies" => TargetType::AllEnemies,
                        "RandomAlly" => TargetType::RandomAlly,
                        "AllAllies" => TargetType::AllAllies,
                        "Owner" => TargetType::Owner,
                        _ => TargetType::RandomEnemy,
                    };
                    if let Err(e) = conn.reducers.ability_create(
                        result.name.clone(),
                        result.description.clone(),
                        target_type,
                        result.effect_script.clone(),
                        0,
                        0,
                        0,
                    ) {
                        warn!("Failed to create ability: {:?}", e);
                    }
                }
                state.breeding_result = None;
                state.breeding_status = RequestStatus::Idle;
                state.breeding_prompt.clear();
            }
            if ui.button("Reject").clicked() {
                state.breeding_result = None;
                state.breeding_status = RequestStatus::Idle;
            }
        });
    }
}

fn assemble_unit_ui(
    ui: &mut egui::Ui,
    state: &mut ResMut<CreateState>,
    content: &GameContent,
    stdb: &StdbConnection,
) {
    ui.heading("Assemble New Unit");
    ui.label("Pick a trigger, abilities, and tier. AI generates the name and visuals.");
    ui.separator();

    // Tier selector
    ui.horizontal(|ui| {
        ui.label("Tier:");
        for t in 1..=5u8 {
            if ui
                .selectable_label(state.unit_tier == t, format!("T{}", t))
                .on_hover_text(format!(
                    "Budget: {}, Abilities: {}",
                    t as i32 * 5,
                    match t {
                        1..=2 => 1,
                        3..=4 => 2,
                        5 => 3,
                        _ => 0,
                    }
                ))
                .clicked()
            {
                state.unit_tier = t;
                state.unit_selected_abilities.clear();
            }
        }
    });

    if state.unit_tier == 0 {
        state.unit_tier = 1;
    }

    let max_abilities = match state.unit_tier {
        1..=2 => 1,
        3..=4 => 2,
        5 => 3,
        _ => 1,
    };
    let budget = state.unit_tier as i32 * 5;

    // Trigger selector
    ui.horizontal(|ui| {
        ui.label("Trigger:");
        egui::ComboBox::from_id_salt("trigger_select")
            .selected_text(*TRIGGERS.get(state.unit_trigger_idx).unwrap_or(&"Select..."))
            .show_ui(ui, |ui| {
                for (i, trigger) in TRIGGERS.iter().enumerate() {
                    ui.selectable_value(&mut state.unit_trigger_idx, i, *trigger);
                }
            });
    });

    // Ability selector
    ui.separator();
    ui.label(format!("Abilities (pick up to {}):", max_abilities));
    for (i, ability) in content.abilities.iter().enumerate() {
        let selected = state.unit_selected_abilities.contains(&i);
        let can_select = selected || state.unit_selected_abilities.len() < max_abilities;

        if ui
            .add_enabled(
                can_select,
                egui::SelectableLabel::new(selected, &ability.name),
            )
            .clicked()
        {
            if selected {
                state.unit_selected_abilities.retain(|&x| x != i);
            } else if state.unit_selected_abilities.len() < max_abilities {
                state.unit_selected_abilities.push(i);
            }
        }
    }

    // Stats
    ui.separator();
    ui.label(format!("Stats (budget: {}):", budget));

    if state.unit_hp == 0 {
        state.unit_hp = budget / 2;
    }
    if state.unit_pwr == 0 {
        state.unit_pwr = budget - state.unit_hp;
    }

    ui.horizontal(|ui| {
        ui.label("HP:");
        if ui
            .add(egui::DragValue::new(&mut state.unit_hp).range(1..=budget - 1))
            .changed()
        {
            state.unit_pwr = (budget - state.unit_hp).max(1);
        }
        ui.label("PWR:");
        if ui
            .add(egui::DragValue::new(&mut state.unit_pwr).range(1..=budget - 1))
            .changed()
        {
            state.unit_hp = (budget - state.unit_pwr).max(1);
        }
        let total = state.unit_hp + state.unit_pwr;
        if total > budget {
            ui.colored_label(
                egui::Color32::RED,
                format!("Over budget! {}/{}", total, budget),
            );
        } else {
            ui.label(format!("{}/{}", total, budget));
        }
    });

    // Description prompt
    ui.separator();
    ui.label("Describe your unit (for AI name + visual generation):");
    ui.text_edit_singleline(&mut state.unit_prompt);

    let can_submit = !state.unit_selected_abilities.is_empty()
        && !state.unit_prompt.is_empty()
        && state.unit_hp + state.unit_pwr <= budget
        && state.unit_status == RequestStatus::Idle;

    if ui
        .add_enabled(can_submit, egui::Button::new("Generate Name & Visual"))
        .clicked()
    {
        if let Some(ref conn) = stdb.conn {
            match conn.reducers.gen_create_unit(state.unit_prompt.clone()) {
                Ok(_) => {
                    state.unit_status = RequestStatus::Pending;
                    info!("Unit generation request sent");
                }
                Err(e) => {
                    state.unit_status = RequestStatus::Error(format!("{:?}", e));
                }
            }
        } else {
            state.unit_status = RequestStatus::Error("Not connected".to_string());
        }
    }

    match &state.unit_status {
        RequestStatus::Pending => {
            ui.spinner();
            ui.label("AI generation request submitted...");
        }
        RequestStatus::Error(e) => {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
            if ui.button("Dismiss").clicked() {
                state.unit_status = RequestStatus::Idle;
            }
        }
        _ => {}
    }

    // Show result
    let unit_result_clone = state.unit_result.clone();
    if let Some(result) = &unit_result_clone {
        ui.separator();
        ui.group(|ui| {
            ui.colored_label(
                tier_color(state.unit_tier),
                format!("T{} {}", state.unit_tier, result.name),
            );
            ui.label(format!("{}hp / {}pwr", state.unit_hp, state.unit_pwr));
            ui.separator();
            ui.label("Painter script:");
            ui.code(&result.painter_script);
        });

        ui.horizontal(|ui| {
            if ui.button("Submit to Incubator").clicked() {
                // Call unit_create reducer
                if let Some(ref conn) = stdb.conn {
                    let trigger = match state.unit_trigger_idx {
                        0 => Trigger::BattleStart,
                        1 => Trigger::TurnEnd,
                        2 => Trigger::BeforeDeath,
                        3 => Trigger::AllyDeath,
                        4 => Trigger::BeforeStrike,
                        5 => Trigger::AfterStrike,
                        6 => Trigger::DamageTaken,
                        7 => Trigger::DamageDealt,
                        _ => Trigger::BeforeStrike,
                    };
                    let ability_ids: Vec<u64> = state
                        .unit_selected_abilities
                        .iter()
                        .filter_map(|&idx| content.abilities.get(idx).map(|a| a.id))
                        .collect();

                    if let Err(e) = conn.reducers.unit_create(
                        result.name.clone(),
                        state.unit_prompt.clone(),
                        state.unit_hp,
                        state.unit_pwr,
                        state.unit_tier,
                        trigger,
                        ability_ids,
                        result.painter_script.clone(),
                    ) {
                        warn!("Failed to create unit: {:?}", e);
                    }
                }
                state.unit_result = None;
                state.unit_status = RequestStatus::Idle;
                state.unit_prompt.clear();
            }
            if ui.button("Regenerate").clicked() {
                state.unit_result = None;
                state.unit_status = RequestStatus::Idle;
            }
        });
    }
}
